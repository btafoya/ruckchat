//! Server-side per-message read-state service.
//!
//! Tracks which messages each user has read so unread badges and the
//! `is:unread` search operator have a single, cross-device source of truth
//! instead of the client-only `localStorage` model this replaces.

use crate::services::channel::ChannelService;
use crate::services::direct_message::DirectMessageService;
use crate::services::events::EventBus;
use ruckchat_domain::{ConversationType, MessageReadRepository};
use ruckchat_id::{ChannelId, MessageId, OrganizationId, UserId};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Dependencies required by [`ReadStateService`].
#[derive(Clone)]
pub struct ReadStateServiceDeps {
    /// Message read-state repository.
    pub reads: Arc<dyn MessageReadRepository + Send + Sync>,
    /// Channel service, reused for its read-visibility checks.
    pub channels: ChannelService,
    /// Direct message service, reused for its membership checks.
    pub direct_messages: DirectMessageService,
    /// Event bus for cross-device read-state sync.
    pub events: Arc<dyn EventBus + Send + Sync>,
}

/// Per-user, per-message read-state operations.
#[derive(Clone)]
pub struct ReadStateService {
    deps: ReadStateServiceDeps,
}

impl ReadStateService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: ReadStateServiceDeps) -> Self {
        Self { deps }
    }

    /// Marks the given messages as read by the caller, then notifies the
    /// caller's other sessions so their unread badges stay in sync.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::NotFound`] or
    /// [`ruckchat_common::Error::Forbidden`] when the caller cannot read the
    /// conversation the messages belong to.
    pub async fn mark_conversation_read(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
        conversation_type: ConversationType,
        message_ids: Vec<MessageId>,
    ) -> ruckchat_common::Result<()> {
        match conversation_type {
            ConversationType::Channel => {
                self.deps
                    .channels
                    .get_channel(caller_id, ChannelId::from_uuid(conversation_id))
                    .await?;
            }
            ConversationType::DirectMessage => {
                self.deps
                    .direct_messages
                    .get_conversation(caller_id, conversation_id)
                    .await?;
            }
        }

        if message_ids.is_empty() {
            return Ok(());
        }

        self.deps.reads.mark_read(caller_id, &message_ids).await?;
        self.deps
            .events
            .publish_read_state_updated(caller_id, conversation_id, &message_ids)
            .await?;
        Ok(())
    }

    /// Returns unread message counts per conversation the caller belongs to
    /// within an organization, covering both channels and DMs.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Forbidden`] when the caller is not an
    /// organization member.
    pub async fn unread_counts(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<HashMap<Uuid, i64>> {
        let channels = self
            .deps
            .channels
            .list_channels_in_organization(caller_id, organization_id)
            .await?;
        let conversations = self
            .deps
            .direct_messages
            .list_conversations_for_user(caller_id, organization_id)
            .await?;

        let mut conversation_ids: Vec<Uuid> = channels.iter().map(|c| c.id.as_uuid()).collect();
        conversation_ids.extend(conversations.iter().map(|c| c.id.as_uuid()));

        self.deps
            .reads
            .unread_counts_by_conversation(caller_id, &conversation_ids)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::authorization::AuthorizationService;
    use crate::services::channel::ChannelServiceDeps;
    use crate::services::direct_message::DirectMessageServiceDeps;
    use crate::services::events::ServerEvent;
    use crate::testing::{
        MockChannelMembershipRepository, MockChannelRepository,
        MockDirectMessageConversationRepository, MockEventBus, MockMessageReadRepository,
        MockMessageRepository, MockOrganizationMembershipRepository,
    };
    use ruckchat_common::Error;
    use ruckchat_domain::{
        Channel, ChannelMembership, ChannelMembershipRepository, ChannelRepository,
        DirectMessageConversation, DirectMessageConversationRepository, Message, MessageRepository,
        OrganizationMembership, OrganizationMembershipRepository, Role, User,
    };
    use ruckchat_id::OrganizationId;

    /// Shared repository handles kept alongside the composed service, so
    /// tests can seed data the same way the real handlers would populate it.
    struct Fixture {
        svc: ReadStateService,
        events: Arc<MockEventBus>,
        messages: Arc<MockMessageRepository>,
        memberships: Arc<MockOrganizationMembershipRepository>,
        channels_repo: Arc<MockChannelRepository>,
        channel_memberships: Arc<MockChannelMembershipRepository>,
        conversations_repo: Arc<MockDirectMessageConversationRepository>,
    }

    fn fixture() -> Fixture {
        let events = Arc::new(MockEventBus::new());
        let messages = Arc::new(MockMessageRepository::new());
        let channel_memberships = Arc::new(MockChannelMembershipRepository::new());
        let memberships = Arc::new(MockOrganizationMembershipRepository::new());
        let channels_repo = Arc::new(MockChannelRepository::new());
        let conversations_repo = Arc::new(MockDirectMessageConversationRepository::new());

        let channels = ChannelService::new(ChannelServiceDeps {
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships.clone(),
            memberships: memberships.clone(),
            authorization: AuthorizationService::new(),
        });
        let direct_messages = DirectMessageService::new(DirectMessageServiceDeps {
            conversations: conversations_repo.clone(),
            memberships: memberships.clone(),
        });
        let reads = Arc::new(MockMessageReadRepository::new(messages.messages_handle()));

        let svc = ReadStateService::new(ReadStateServiceDeps {
            reads,
            channels,
            direct_messages,
            events: events.clone(),
        });
        Fixture {
            svc,
            events,
            messages,
            memberships,
            channels_repo,
            channel_memberships,
            conversations_repo,
        }
    }

    async fn seed_channel(fx: &Fixture) -> (UserId, OrganizationId, ChannelId, MessageId) {
        let author = User::new("author@example.com", "Author", "hash").unwrap();
        let org_id = OrganizationId::new();
        fx.memberships
            .create(&OrganizationMembership::new(author.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let channel = Channel::new(org_id, "general", author.id, false).unwrap();
        fx.channels_repo.create(&channel).await.unwrap();
        fx.channel_memberships
            .create(&ChannelMembership::new(author.id, channel.id).unwrap())
            .await
            .unwrap();

        let message = Message::new(
            channel.id.as_uuid(),
            ConversationType::Channel,
            author.id,
            "hello",
            None,
            vec![],
        )
        .unwrap();
        fx.messages.create(&message).await.unwrap();

        (author.id, org_id, channel.id, message.id)
    }

    #[tokio::test]
    async fn member_can_mark_channel_messages_read_and_they_no_longer_count_as_unread() {
        let fx = fixture();
        let (_author_id, org_id, channel_id, message_id) = seed_channel(&fx).await;

        let other = User::new("other@example.com", "Other", "hash").unwrap();
        fx.memberships
            .create(&OrganizationMembership::new(other.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        fx.channel_memberships
            .create(&ChannelMembership::new(other.id, channel_id).unwrap())
            .await
            .unwrap();

        let before = fx.svc.unread_counts(other.id, org_id).await.unwrap();
        assert_eq!(before.get(&channel_id.as_uuid()), Some(&1));

        fx.svc
            .mark_conversation_read(
                other.id,
                channel_id.as_uuid(),
                ConversationType::Channel,
                vec![message_id],
            )
            .await
            .unwrap();

        let after = fx.svc.unread_counts(other.id, org_id).await.unwrap();
        assert_eq!(after.get(&channel_id.as_uuid()), None);
    }

    #[tokio::test]
    async fn own_authored_messages_never_count_as_unread() {
        let fx = fixture();
        let (author_id, org_id, channel_id, _message_id) = seed_channel(&fx).await;

        let counts = fx.svc.unread_counts(author_id, org_id).await.unwrap();
        assert_eq!(counts.get(&channel_id.as_uuid()), None);
    }

    #[tokio::test]
    async fn marking_read_emits_read_state_updated_event() {
        let fx = fixture();
        let (_author_id, org_id, channel_id, message_id) = seed_channel(&fx).await;
        let other = User::new("other@example.com", "Other", "hash").unwrap();
        fx.memberships
            .create(&OrganizationMembership::new(other.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        fx.channel_memberships
            .create(&ChannelMembership::new(other.id, channel_id).unwrap())
            .await
            .unwrap();

        fx.svc
            .mark_conversation_read(
                other.id,
                channel_id.as_uuid(),
                ConversationType::Channel,
                vec![message_id],
            )
            .await
            .unwrap();

        assert!(fx.events.events().iter().any(|e| matches!(
            e,
            ServerEvent::ReadStateUpdated { message_ids, .. } if message_ids == &vec![message_id]
        )));
    }

    #[tokio::test]
    async fn non_member_cannot_mark_private_channel_read() {
        let fx = fixture();
        let author = User::new("author@example.com", "Author", "hash").unwrap();
        let org_id = OrganizationId::new();
        fx.memberships
            .create(&OrganizationMembership::new(author.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let private_channel = Channel::new(org_id, "secret", author.id, true).unwrap();
        fx.channels_repo.create(&private_channel).await.unwrap();
        let message = Message::new(
            private_channel.id.as_uuid(),
            ConversationType::Channel,
            author.id,
            "shh",
            None,
            vec![],
        )
        .unwrap();
        fx.messages.create(&message).await.unwrap();

        let outsider = User::new("outsider@example.com", "Outsider", "hash").unwrap();
        fx.memberships
            .create(&OrganizationMembership::new(outsider.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = fx
            .svc
            .mark_conversation_read(
                outsider.id,
                private_channel.id.as_uuid(),
                ConversationType::Channel,
                vec![message.id],
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn dm_participant_can_mark_read() {
        let fx = fixture();
        let a = User::new("a@example.com", "A", "hash").unwrap();
        let b = User::new("b@example.com", "B", "hash").unwrap();
        let org_id = OrganizationId::new();
        for user in [&a, &b] {
            fx.memberships
                .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
                .await
                .unwrap();
        }
        let dm = DirectMessageConversation::new(org_id, [a.id, b.id]).unwrap();
        fx.conversations_repo.create(&dm).await.unwrap();
        let message = Message::new(
            dm.id.as_uuid(),
            ConversationType::DirectMessage,
            a.id,
            "hi",
            None,
            vec![],
        )
        .unwrap();
        fx.messages.create(&message).await.unwrap();

        fx.svc
            .mark_conversation_read(
                b.id,
                dm.id.as_uuid(),
                ConversationType::DirectMessage,
                vec![message.id],
            )
            .await
            .unwrap();

        let counts = fx.svc.unread_counts(b.id, org_id).await.unwrap();
        assert_eq!(counts.get(&dm.id.as_uuid()), None);
    }
}
