//! Message service.

use crate::services::authorization::AuthorizationService;
use crate::services::dto::{EditMessageRequest, Pagination, PostMessageRequest};
use crate::services::events::EventBus;
use ruckchat_common::Error;
use ruckchat_domain::{
    ChannelMembershipRepository, ChannelRepository, ConversationType, Message, MessageRepository,
    OrganizationMembershipRepository,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, MessageId, UserId};
use std::sync::Arc;
use uuid::Uuid;

/// Dependencies required by [`MessageService`].
#[derive(Clone)]
pub struct MessageServiceDeps {
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
    /// Authorization service.
    pub authorization: AuthorizationService,
    /// Event bus for real-time updates.
    pub events: Arc<dyn EventBus + Send + Sync>,
}

/// Message posting, editing, deletion, and history.
#[derive(Clone)]
pub struct MessageService {
    deps: MessageServiceDeps,
}

impl MessageService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: MessageServiceDeps) -> Self {
        Self { deps }
    }

    /// Posts a message to a channel or DM conversation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller cannot post,
    /// [`Error::NotFound`] when the conversation does not exist, and
    /// [`Error::Validation`] for invalid content.
    pub async fn post_message(
        &self,
        caller_id: UserId,
        request: PostMessageRequest,
    ) -> ruckchat_common::Result<Message> {
        match request.conversation_type {
            ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(request.conversation_id);
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

                self.deps.authorization.require_can_post_in_channel(
                    &channel,
                    caller_membership.as_ref(),
                    channel_membership.as_ref(),
                )?;

                let message = Message::new(
                    request.conversation_id,
                    request.conversation_type,
                    caller_id,
                    request.content,
                    request.parent_id,
                )?;
                self.deps.messages.create(&message).await?;
                self.deps.events.publish_message_created(&message).await?;
                Ok(message)
            }
            ConversationType::DirectMessage => {
                let conversation_id =
                    DirectMessageConversationId::from_uuid(request.conversation_id);
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
                        "must be a conversation member to post".into(),
                    ));
                }

                let message = Message::new(
                    request.conversation_id,
                    request.conversation_type,
                    caller_id,
                    request.content,
                    request.parent_id,
                )?;
                self.deps.messages.create(&message).await?;
                self.deps.events.publish_message_created(&message).await?;
                Ok(message)
            }
        }
    }

    /// Edits an existing message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message does not exist,
    /// [`Error::Forbidden`] when the caller lacks permission, and
    /// [`Error::Validation`] for invalid content.
    pub async fn edit_message(
        &self,
        caller_id: UserId,
        message_id: MessageId,
        request: EditMessageRequest,
    ) -> ruckchat_common::Result<Message> {
        let mut message = self
            .deps
            .messages
            .by_id(message_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;

        if message.is_deleted() {
            return Err(Error::Forbidden("cannot edit a deleted message".into()));
        }

        let role = self.organization_role(caller_id, &message).await?;
        self.deps
            .authorization
            .require_can_edit_message(&message, caller_id, role)?;

        message.edit(request.content)?;
        self.deps.messages.update(&message).await?;
        self.deps.events.publish_message_updated(&message).await?;
        Ok(message)
    }

    /// Soft-deletes a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message does not exist or
    /// [`Error::Forbidden`] when the caller lacks permission.
    pub async fn delete_message(
        &self,
        caller_id: UserId,
        message_id: MessageId,
    ) -> ruckchat_common::Result<()> {
        let mut message = self
            .deps
            .messages
            .by_id(message_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;

        let role = self.organization_role(caller_id, &message).await?;
        self.deps
            .authorization
            .require_can_delete_message(&message, caller_id, role)?;

        message.delete();
        self.deps.messages.update(&message).await?;
        self.deps.events.publish_message_deleted(&message).await?;
        Ok(())
    }

    /// Returns message history for a conversation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller cannot read the conversation
    /// or [`Error::NotFound`] when the channel does not exist.
    pub async fn get_history(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
        conversation_type: ConversationType,
        pagination: Pagination,
    ) -> ruckchat_common::Result<Vec<Message>> {
        self.require_can_read(caller_id, conversation_id, conversation_type)
            .await?;

        let pagination = pagination.normalized();
        self.deps
            .messages
            .list_by_conversation(conversation_id, pagination.limit, pagination.offset)
            .await
    }

    /// Returns replies to a parent message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the parent does not exist or
    /// [`Error::Forbidden`] when the caller cannot read the conversation.
    pub async fn get_thread_replies(
        &self,
        caller_id: UserId,
        parent_id: MessageId,
        pagination: Pagination,
    ) -> ruckchat_common::Result<Vec<Message>> {
        let parent = self
            .deps
            .messages
            .by_id(parent_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;

        self.require_can_read(caller_id, parent.conversation_id, parent.conversation_type)
            .await?;

        let all = self
            .deps
            .messages
            .list_by_conversation(parent.conversation_id, 1000, 0)
            .await?;
        let replies: Vec<Message> = all
            .into_iter()
            .filter(|m| m.parent_id == Some(parent_id) && !m.is_deleted())
            .collect();

        let pagination = pagination.normalized();
        let offset = pagination.offset as usize;
        let limit = pagination.limit as usize;
        let paginated = replies.into_iter().skip(offset).take(limit).collect();
        Ok(paginated)
    }

    async fn organization_role(
        &self,
        caller_id: UserId,
        message: &Message,
    ) -> ruckchat_common::Result<ruckchat_domain::Role> {
        match message.conversation_type {
            ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(message.conversation_id);
                let channel = self
                    .deps
                    .channels
                    .by_id(channel_id)
                    .await?
                    .ok_or_else(|| Error::NotFound("channel".into()))?;
                let membership = self
                    .deps
                    .memberships
                    .by_ids(caller_id, channel.organization_id)
                    .await?;
                Ok(membership
                    .map(|m| m.role)
                    .unwrap_or(ruckchat_domain::Role::Member))
            }
            ConversationType::DirectMessage => {
                let conversation_id =
                    DirectMessageConversationId::from_uuid(message.conversation_id);
                let conversation = self
                    .deps
                    .conversations
                    .by_id(conversation_id)
                    .await?
                    .ok_or_else(|| Error::NotFound("dm conversation".into()))?;
                let membership = self
                    .deps
                    .memberships
                    .by_ids(caller_id, conversation.organization_id)
                    .await?;
                Ok(membership
                    .map(|m| m.role)
                    .unwrap_or(ruckchat_domain::Role::Member))
            }
        }
    }

    async fn require_can_read(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
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
                self.deps.authorization.require_can_read_channel(
                    &channel,
                    caller_membership.as_ref(),
                    channel_membership.as_ref(),
                )?;
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
                        "must be a conversation member to read".into(),
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
    use crate::services::authorization::AuthorizationService;
    use crate::services::dto::{EditMessageRequest, PostMessageRequest};
    use crate::testing::{
        MockChannelMembershipRepository, MockChannelRepository,
        MockDirectMessageConversationRepository, MockEventBus, MockMessageRepository,
        MockOrganizationMembershipRepository,
    };
    use ruckchat_domain::{
        Channel, ChannelMembership, ConversationType, DirectMessageConversation,
        OrganizationMembership, Role, User,
    };
    use ruckchat_id::{ChannelId, OrganizationId, UserId};
    use std::sync::Arc;

    fn service() -> (MessageService, Arc<MockEventBus>) {
        let events = Arc::new(MockEventBus::new());
        let svc = MessageService::new(MessageServiceDeps {
            messages: Arc::new(MockMessageRepository::new()),
            channels: Arc::new(MockChannelRepository::new()),
            channel_memberships: Arc::new(MockChannelMembershipRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
            conversations: Arc::new(MockDirectMessageConversationRepository::new()),
            authorization: AuthorizationService::new(),
            events: events.clone(),
        });
        (svc, events)
    }

    async fn seed_channel(
        svc: &MessageService,
        is_private: bool,
    ) -> (UserId, OrganizationId, ChannelId) {
        let user = User::new("author@example.com", "Author", "hash").unwrap();
        let org_id = OrganizationId::new();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let channel = Channel::new(org_id, "general", user.id, is_private).unwrap();
        svc.deps.channels.create(&channel).await.unwrap();
        svc.deps
            .channel_memberships
            .create(&ChannelMembership::new(user.id, channel.id).unwrap())
            .await
            .unwrap();
        (user.id, org_id, channel.id)
    }

    #[tokio::test]
    async fn post_message_in_public_channel() {
        let (svc, _events) = service();
        let (author_id, _, channel_id) = seed_channel(&svc, false).await;
        let msg = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content: "hello".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(msg.content, "hello");
    }

    #[tokio::test]
    async fn posting_message_emits_created_event() {
        let (svc, events) = service();
        let (author_id, _, channel_id) = seed_channel(&svc, false).await;
        let msg = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content: "hello".into(),
                },
            )
            .await
            .unwrap();

        assert!(events.events().iter().any(|e| matches!(
            e,
            crate::services::events::ServerEvent::MessageCreated { message } if message.id == msg.id
        )));
    }

    #[tokio::test]
    async fn non_member_cannot_post_in_channel() {
        let (svc, _events) = service();
        let (_author_id, org_id, channel_id) = seed_channel(&svc, false).await;
        let outsider = User::new("outsider@example.com", "Outsider", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(outsider.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .post_message(
                outsider.id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content: "hello".into(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn author_can_edit_message() {
        let (svc, _events) = service();
        let (author_id, _, channel_id) = seed_channel(&svc, false).await;
        let msg = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content: "hello".into(),
                },
            )
            .await
            .unwrap();

        let edited = svc
            .edit_message(
                author_id,
                msg.id,
                EditMessageRequest {
                    content: "hello world".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(edited.content, "hello world");
    }

    #[tokio::test]
    async fn outsider_cannot_edit_message() {
        let (svc, _events) = service();
        let (author_id, org_id, channel_id) = seed_channel(&svc, false).await;
        let msg = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content: "hello".into(),
                },
            )
            .await
            .unwrap();

        let outsider = User::new("outsider@example.com", "Outsider", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(outsider.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .edit_message(
                outsider.id,
                msg.id,
                EditMessageRequest {
                    content: "hacked".into(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn post_message_in_dm_requires_membership() {
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

        let outsider = User::new("c@example.com", "C", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(outsider.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .post_message(
                outsider.id,
                PostMessageRequest {
                    conversation_id: dm.id.as_uuid(),
                    conversation_type: ConversationType::DirectMessage,
                    parent_id: None,
                    content: "hello".into(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
