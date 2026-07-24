//! Reaction service.

use crate::services::events::EventBus;
use ruckchat_common::Error;
use ruckchat_domain::{
    ChannelMembershipRepository, ChannelRepository, ConversationType, MessageRepository,
    OrganizationMembershipRepository, Reaction, ReactionRepository,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, MessageId, UserId};
use std::sync::Arc;

/// Dependencies required by [`ReactionService`].
#[derive(Clone)]
pub struct ReactionServiceDeps {
    /// Reaction repository.
    pub reactions: Arc<dyn ReactionRepository + Send + Sync>,
    /// Message repository.
    pub messages: Arc<dyn MessageRepository + Send + Sync>,
    /// Channel repository.
    pub channels: Arc<dyn ChannelRepository + Send + Sync>,
    /// Channel membership repository.
    pub channel_memberships: Arc<dyn ChannelMembershipRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn OrganizationMembershipRepository + Send + Sync>,
    /// DM conversation repository.
    pub conversations: Arc<dyn ruckchat_domain::DirectMessageConversationRepository + Send + Sync>,
    /// Event bus for real-time updates.
    pub events: Arc<dyn EventBus + Send + Sync>,
}

/// Reaction operations.
#[derive(Clone)]
pub struct ReactionService {
    deps: ReactionServiceDeps,
}

impl ReactionService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: ReactionServiceDeps) -> Self {
        Self { deps }
    }

    /// Adds a reaction to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message does not exist,
    /// [`Error::Forbidden`] when the caller cannot read the conversation, and
    /// [`Error::Validation`] for an empty emoji.
    pub async fn add_reaction(
        &self,
        caller_id: UserId,
        message_id: MessageId,
        emoji: String,
    ) -> ruckchat_common::Result<Reaction> {
        let message = self
            .deps
            .messages
            .by_id(message_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;

        self.require_can_read_conversation(
            caller_id,
            message.conversation_id,
            message.conversation_type,
        )
        .await?;

        let reaction = Reaction::new(message_id, caller_id, emoji)?;
        self.deps.reactions.create(&reaction).await?;
        self.deps.events.publish_reaction_added(&reaction).await?;
        Ok(reaction)
    }

    /// Removes the caller's reaction from a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message or reaction does not exist or
    /// [`Error::Forbidden`] when the caller cannot read the conversation.
    pub async fn remove_reaction(
        &self,
        caller_id: UserId,
        message_id: MessageId,
        emoji: &str,
    ) -> ruckchat_common::Result<()> {
        let message = self
            .deps
            .messages
            .by_id(message_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;

        self.require_can_read_conversation(
            caller_id,
            message.conversation_id,
            message.conversation_type,
        )
        .await?;

        self.deps
            .reactions
            .delete(message_id, caller_id, emoji)
            .await?;
        self.deps
            .events
            .publish_reaction_removed(message_id, caller_id, emoji)
            .await?;
        Ok(())
    }

    async fn require_can_read_conversation(
        &self,
        caller_id: UserId,
        conversation_id: uuid::Uuid,
        conversation_type: ConversationType,
    ) -> ruckchat_common::Result<()> {
        match conversation_type {
            ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(conversation_id);
                let channel = self
                    .deps
                    .channels
                    .by_id(channel_id)
                    .await?
                    .ok_or_else(|| Error::NotFound("channel".into()))?;
                let caller_membership = self
                    .deps
                    .memberships
                    .by_ids(caller_id, channel.organization_id)
                    .await?;
                let channel_membership = self
                    .deps
                    .channel_memberships
                    .by_ids(caller_id, channel_id)
                    .await?;

                if caller_membership.is_none() {
                    return Err(Error::Forbidden("must be an organization member".into()));
                }
                if channel.is_private && channel_membership.is_none() {
                    return Err(Error::Forbidden(
                        "must be a member of the private channel".into(),
                    ));
                }
            }
            ConversationType::DirectMessage => {
                let conversation_id = DirectMessageConversationId::from_uuid(conversation_id);
                let conversation = self
                    .deps
                    .conversations
                    .by_id(conversation_id)
                    .await?
                    .ok_or_else(|| Error::NotFound("dm conversation".into()))?;
                let caller_membership = self
                    .deps
                    .memberships
                    .by_ids(caller_id, conversation.organization_id)
                    .await?;
                if caller_membership.is_none() {
                    return Err(Error::Forbidden("must be an organization member".into()));
                }
                if !conversation.member_ids.contains(&caller_id) {
                    return Err(Error::Forbidden(
                        "must be a conversation member to react".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::events::ServerEvent;
    use crate::testing::{
        MockChannelMembershipRepository, MockChannelRepository,
        MockDirectMessageConversationRepository, MockEventBus, MockMessageRepository,
        MockOrganizationMembershipRepository, MockReactionRepository,
    };
    use ruckchat_domain::{
        Channel, ChannelMembership, ConversationType, DirectMessageConversation, Message,
        OrganizationMembership, Role, User,
    };
    use ruckchat_id::{OrganizationId, UserId};
    use std::sync::Arc;

    fn service() -> (ReactionService, Arc<MockEventBus>) {
        let events = Arc::new(MockEventBus::new());
        let svc = ReactionService::new(ReactionServiceDeps {
            reactions: Arc::new(MockReactionRepository::new()),
            messages: Arc::new(MockMessageRepository::new()),
            channels: Arc::new(MockChannelRepository::new()),
            channel_memberships: Arc::new(MockChannelMembershipRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
            conversations: Arc::new(MockDirectMessageConversationRepository::new()),
            events: events.clone(),
        });
        (svc, events)
    }

    async fn seed_channel_and_message(
        svc: &ReactionService,
    ) -> (
        UserId,
        OrganizationId,
        ruckchat_id::ChannelId,
        ruckchat_id::MessageId,
    ) {
        let user = User::new("author@example.com", "Author", "hash").unwrap();
        let org_id = OrganizationId::new();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let channel = Channel::new(org_id, "general", user.id, false).unwrap();
        svc.deps.channels.create(&channel).await.unwrap();
        svc.deps
            .channel_memberships
            .create(&ChannelMembership::new(user.id, channel.id).unwrap())
            .await
            .unwrap();
        let message = Message::new(
            channel.id.as_uuid(),
            ConversationType::Channel,
            user.id,
            "hello",
            None,
            vec![],
        )
        .unwrap();
        svc.deps.messages.create(&message).await.unwrap();
        (user.id, org_id, channel.id, message.id)
    }

    #[tokio::test]
    async fn member_can_add_reaction_to_channel_message() {
        let (svc, _events) = service();
        let (user_id, _, _, message_id) = seed_channel_and_message(&svc).await;

        let reaction = svc
            .add_reaction(user_id, message_id, "👍".into())
            .await
            .unwrap();
        assert_eq!(reaction.emoji, "👍");
    }

    #[tokio::test]
    async fn adding_reaction_emits_event() {
        let (svc, events) = service();
        let (user_id, _, _, message_id) = seed_channel_and_message(&svc).await;

        svc.add_reaction(user_id, message_id, "👍".into())
            .await
            .unwrap();

        assert!(events.events().iter().any(|e| matches!(
            e,
            ServerEvent::ReactionAdded { reaction } if reaction.emoji == "👍"
        )));
    }

    #[tokio::test]
    async fn non_member_cannot_add_reaction_to_private_channel() {
        let (svc, _events) = service();
        let (author_id, org_id, _, _message_id) = seed_channel_and_message(&svc).await;
        let channel = Channel::new(org_id, "secret", author_id, true).unwrap();
        svc.deps.channels.create(&channel).await.unwrap();
        let message = Message::new(
            channel.id.as_uuid(),
            ConversationType::Channel,
            author_id,
            "secret",
            None,
            vec![],
        )
        .unwrap();
        svc.deps.messages.create(&message).await.unwrap();

        let outsider = User::new("outsider@example.com", "Outsider", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(outsider.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .add_reaction(outsider.id, message.id, "👍".into())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn member_can_remove_own_reaction() {
        let (svc, events) = service();
        let (user_id, _, _, message_id) = seed_channel_and_message(&svc).await;

        svc.add_reaction(user_id, message_id, "👍".into())
            .await
            .unwrap();
        svc.remove_reaction(user_id, message_id, "👍")
            .await
            .unwrap();

        assert!(events.events().iter().any(|e| matches!(
            e,
            ServerEvent::ReactionRemoved { emoji, .. } if emoji == "👍"
        )));
    }

    #[tokio::test]
    async fn dm_member_can_react() {
        let (svc, _events) = service();
        let a = User::new("a@example.com", "A", "hash").unwrap();
        let b = User::new("b@example.com", "B", "hash").unwrap();
        let org_id = OrganizationId::new();
        for user in [&a, &b] {
            svc.deps
                .memberships
                .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
                .await
                .unwrap();
        }
        let dm = DirectMessageConversation::new(org_id, [a.id, b.id]).unwrap();
        svc.deps.conversations.create(&dm).await.unwrap();
        let message = Message::new(
            dm.id.as_uuid(),
            ConversationType::DirectMessage,
            a.id,
            "hello",
            None,
            vec![],
        )
        .unwrap();
        svc.deps.messages.create(&message).await.unwrap();

        let reaction = svc
            .add_reaction(b.id, message.id, "👋".into())
            .await
            .unwrap();
        assert_eq!(reaction.emoji, "👋");
    }
}
