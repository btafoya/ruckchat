//! Message service.

use crate::services::authorization::AuthorizationService;
use crate::services::dto::{EditMessageRequest, Pagination, PostMessageRequest};
use crate::services::events::EventBus;
use ruckchat_common::Error;
use ruckchat_domain::{
    ChannelMembershipRepository, ChannelRepository, ConversationType, Message, MessageRepository,
    OrganizationMembershipRepository, UserRepository,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, MessageId, OrganizationId, UserId};
use std::collections::HashSet;
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
    /// User repository for resolving mention targets.
    pub users: Arc<dyn UserRepository + Send + Sync>,
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

                let mentioned_user_ids = self
                    .validate_mentions(channel.organization_id, &request.content)
                    .await?;

                let message = Message::new(
                    request.conversation_id,
                    request.conversation_type,
                    caller_id,
                    request.content,
                    request.parent_id,
                    mentioned_user_ids,
                )?;
                self.deps.messages.create(&message).await?;
                let message = self
                    .deps
                    .messages
                    .by_id(message.id)
                    .await?
                    .ok_or_else(|| Error::Internal("created message disappeared".into()))?;
                self.deps.events.publish_message_created(&message).await?;
                self.publish_mentions(&message).await?;
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

                let mentioned_user_ids = self
                    .validate_mentions(conversation.organization_id, &request.content)
                    .await?;

                let message = Message::new(
                    request.conversation_id,
                    request.conversation_type,
                    caller_id,
                    request.content,
                    request.parent_id,
                    mentioned_user_ids,
                )?;
                self.deps.messages.create(&message).await?;
                let message = self
                    .deps
                    .messages
                    .by_id(message.id)
                    .await?
                    .ok_or_else(|| Error::Internal("created message disappeared".into()))?;
                self.deps.events.publish_message_created(&message).await?;
                self.publish_mentions(&message).await?;
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

        let organization_id = self.conversation_organization_id(&message).await?;

        message.edit(request.content)?;
        message.mentioned_user_ids = self
            .validate_mentions(organization_id, &message.content)
            .await?;
        self.deps.messages.update(&message).await?;
        self.deps.events.publish_message_updated(&message).await?;
        self.publish_mentions(&message).await?;
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

    /// Searches message content visible to the caller within an organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn search_messages(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        query: &str,
        pagination: Pagination,
    ) -> ruckchat_common::Result<Vec<Message>> {
        let caller_membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?;
        if caller_membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        let pagination = pagination.normalized();
        if pagination.limit == 0 {
            return Ok(Vec::new());
        }
        self.deps
            .messages
            .search(
                caller_id,
                organization_id,
                query,
                pagination.limit,
                pagination.offset,
            )
            .await
    }

    /// Loads a single message visible to the caller.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message does not exist or is deleted,
    /// and [`Error::Forbidden`] when the caller cannot read the conversation.
    pub async fn get_message(
        &self,
        caller_id: UserId,
        message_id: MessageId,
    ) -> ruckchat_common::Result<Message> {
        let message = self
            .deps
            .messages
            .by_id(message_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;
        if message.is_deleted() {
            return Err(Error::NotFound("message".into()));
        }
        self.require_can_read(
            caller_id,
            message.conversation_id,
            message.conversation_type,
        )
        .await?;
        Ok(message)
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

    async fn conversation_organization_id(
        &self,
        message: &Message,
    ) -> ruckchat_common::Result<OrganizationId> {
        match message.conversation_type {
            ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(message.conversation_id);
                let channel = self
                    .deps
                    .channels
                    .by_id(channel_id)
                    .await?
                    .ok_or_else(|| Error::NotFound("channel".into()))?;
                Ok(channel.organization_id)
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
                Ok(conversation.organization_id)
            }
        }
    }

    async fn validate_mentions(
        &self,
        organization_id: OrganizationId,
        content: &str,
    ) -> ruckchat_common::Result<Vec<UserId>> {
        let ids = extract_mention_user_ids(content);
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let members = self
            .deps
            .memberships
            .list_by_organization(organization_id)
            .await?;
        let member_ids: HashSet<UserId> = members.into_iter().map(|m| m.user_id).collect();

        let mut unique = Vec::with_capacity(ids.len());
        let mut seen = HashSet::new();
        for id in ids {
            if !member_ids.contains(&id) {
                return Err(Error::validation(format!(
                    "mentioned user {id} is not an organization member"
                )));
            }
            if seen.insert(id) {
                unique.push(id);
            }
        }

        Ok(unique)
    }

    async fn publish_mentions(&self, message: &Message) -> ruckchat_common::Result<()> {
        for user_id in &message.mentioned_user_ids {
            if *user_id != message.author_id {
                self.deps.events.publish_mention(*user_id, message).await?;
            }
        }
        Ok(())
    }
}

fn extract_mention_user_ids(content: &str) -> Vec<UserId> {
    let value: serde_json::Value = serde_json::from_str(content).unwrap_or_default();
    let mut ids = Vec::new();
    extract_mention_ids_from_value(&value, &mut ids);
    ids
}

fn extract_mention_ids_from_value(value: &serde_json::Value, ids: &mut Vec<UserId>) {
    if let Some(obj) = value.as_object()
        && obj.get("type").and_then(|t| t.as_str()) == Some("mention")
        && let Some(id) = obj
            .get("attrs")
            .and_then(|attrs| attrs.get("id"))
            .and_then(|id| id.as_str())
        && let Ok(uuid) = Uuid::parse_str(id)
    {
        ids.push(UserId::from_uuid(uuid));
    }

    if let Some(children) = value.get("content").and_then(|c| c.as_array()) {
        for child in children {
            extract_mention_ids_from_value(child, ids);
        }
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
        MockOrganizationMembershipRepository, MockUserRepository,
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
            users: Arc::new(MockUserRepository::new()),
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

    fn mention_content(mentions: &[(UserId, &str)]) -> String {
        let mut nodes = Vec::new();
        for (id, label) in mentions {
            nodes.push(serde_json::json!({
                "type": "mention",
                "attrs": {
                    "id": id.to_string(),
                    "label": label,
                }
            }));
        }
        serde_json::json!({
            "type": "doc",
            "content": nodes,
        })
        .to_string()
    }

    #[test]
    fn extract_mention_user_ids_parses_prosemirror_nodes() {
        let a = UserId::new();
        let b = UserId::new();
        let content = mention_content(&[(a, "Alice"), (b, "Bob")]);
        let ids = extract_mention_user_ids(&content);
        assert_eq!(ids, vec![a, b]);
    }

    #[tokio::test]
    async fn post_message_stores_mentions_and_emits_events() {
        let (svc, events) = service();
        let (author_id, org_id, channel_id) = seed_channel(&svc, false).await;
        let mentioned = User::new("mentioned@example.com", "Mentioned", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(mentioned.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let content = mention_content(&[(mentioned.id, "Mentioned")]);
        let msg = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content,
                },
            )
            .await
            .unwrap();

        assert_eq!(msg.mentioned_user_ids, vec![mentioned.id]);
        assert!(events.events().iter().any(|e| matches!(
            e,
            crate::services::events::ServerEvent::Mention { user_id, message }
                if *user_id == mentioned.id && message.id == msg.id
        )));
    }

    #[tokio::test]
    async fn post_message_rejects_non_member_mention() {
        let (svc, _events) = service();
        let (author_id, _org_id, channel_id) = seed_channel(&svc, false).await;
        let outsider = UserId::new();

        let content = mention_content(&[(outsider, "Outsider")]);
        let err = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { message: _ }));
    }

    #[tokio::test]
    async fn edit_message_recomputes_mentions() {
        let (svc, _events) = service();
        let (author_id, org_id, channel_id) = seed_channel(&svc, false).await;
        let first = User::new("first@example.com", "First", "hash").unwrap();
        let second = User::new("second@example.com", "Second", "hash").unwrap();
        for user in [&first, &second] {
            svc.deps
                .memberships
                .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
                .await
                .unwrap();
        }

        let msg = svc
            .post_message(
                author_id,
                PostMessageRequest {
                    conversation_id: channel_id.as_uuid(),
                    conversation_type: ConversationType::Channel,
                    parent_id: None,
                    content: mention_content(&[(first.id, "First")]),
                },
            )
            .await
            .unwrap();
        assert_eq!(msg.mentioned_user_ids, vec![first.id]);

        let edited = svc
            .edit_message(
                author_id,
                msg.id,
                EditMessageRequest {
                    content: mention_content(&[(second.id, "Second")]),
                },
            )
            .await
            .unwrap();
        assert_eq!(edited.mentioned_user_ids, vec![second.id]);
    }
}
