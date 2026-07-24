//! MCP service layer.
//!
//! [`McpService`] exposes a small, authorized surface for the MCP server. It
//! delegates to the existing REST/WebSocket service layer so that MCP tools
//! inherit the same organization, channel, and DM membership checks.

use crate::services::{
    channel::ChannelService, direct_message::DirectMessageService, dto::Pagination,
    message::MessageService, organization::OrganizationService, user::UserService,
};
use ruckchat_common::Error;
use ruckchat_domain::{
    Channel, ConversationType, DirectMessageConversation, Message, Organization, User,
};
use ruckchat_id::{ChannelId, MessageId, OrganizationId, UserId};
use std::sync::Arc;
use uuid::Uuid;

/// Result of a requested `post_message` operation.
#[derive(Debug, Clone)]
pub enum PostMessageResult {
    /// The message was posted.
    Posted(Message),
    /// Confirmation is required before posting.
    ConfirmationRequired {
        /// Target conversation.
        conversation_id: Uuid,
        /// Kind of conversation.
        conversation_type: ConversationType,
        /// Message content.
        content: String,
        /// Optional thread parent.
        parent_id: Option<MessageId>,
    },
}

/// Dependencies required by [`McpService`].
#[derive(Clone)]
pub struct McpServiceDeps {
    /// Channel service.
    pub channels: ChannelService,
    /// Direct message service.
    pub direct_messages: DirectMessageService,
    /// Message service.
    pub messages: MessageService,
    /// User service.
    pub users: UserService,
    /// Organization service.
    pub organizations: OrganizationService,
    /// Organization membership repository for cross-cutting visibility checks.
    pub memberships: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync>,
}

/// MCP tool logic backed by existing RuckChat services.
#[derive(Clone)]
pub struct McpService {
    deps: McpServiceDeps,
    require_confirmation: bool,
}

impl McpService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: McpServiceDeps, require_confirmation: bool) -> Self {
        Self {
            deps,
            require_confirmation,
        }
    }

    /// Lists channels visible to the caller in an organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn list_channels(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<Vec<Channel>> {
        self.deps
            .channels
            .list_channels_in_organization(caller_id, organization_id)
            .await
    }

    /// Lists DM conversations for the caller in an organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn list_direct_messages(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<Vec<DirectMessageConversation>> {
        self.deps
            .direct_messages
            .list_conversations_for_user(caller_id, organization_id)
            .await
    }

    /// Returns recent messages in a conversation the caller can read.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller cannot read the conversation
    /// or [`Error::NotFound`] when the conversation does not exist.
    pub async fn get_messages(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
        conversation_type: ConversationType,
        pagination: Pagination,
    ) -> ruckchat_common::Result<Vec<Message>> {
        self.deps
            .messages
            .get_history(caller_id, conversation_id, conversation_type, pagination)
            .await
    }

    /// Searches message content visible to the caller in an organization.
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
        if query.is_empty() {
            return Err(Error::validation("query must not be empty"));
        }
        self.deps
            .messages
            .search_messages(caller_id, organization_id, query, pagination)
            .await
    }

    /// Posts a message on behalf of the caller, optionally requiring confirmation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller cannot post,
    /// [`Error::NotFound`] when the conversation does not exist, and
    /// [`Error::Validation`] for invalid content.
    pub async fn post_message(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
        conversation_type: ConversationType,
        content: String,
        parent_id: Option<MessageId>,
        confirmed: bool,
    ) -> ruckchat_common::Result<PostMessageResult> {
        if self.require_confirmation && !confirmed {
            return Ok(PostMessageResult::ConfirmationRequired {
                conversation_id,
                conversation_type,
                content,
                parent_id,
            });
        }

        let request = crate::services::dto::PostMessageRequest {
            conversation_id,
            conversation_type,
            content,
            parent_id,
        };
        let message = self.deps.messages.post_message(caller_id, request).await?;
        Ok(PostMessageResult::Posted(message))
    }

    /// Loads a user profile visible to the caller.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the user does not exist or is not visible,
    /// and [`Error::Forbidden`] when the caller cannot view the profile.
    pub async fn get_user_profile(
        &self,
        caller_id: UserId,
        user_id: UserId,
    ) -> ruckchat_common::Result<User> {
        if caller_id == user_id {
            return self.deps.users.get_profile(user_id).await;
        }

        let caller_orgs = self.deps.memberships.list_by_user(caller_id).await?;
        let target_orgs = self.deps.memberships.list_by_user(user_id).await?;
        let shared = caller_orgs.iter().any(|caller| {
            target_orgs
                .iter()
                .any(|target| target.organization_id == caller.organization_id)
        });
        if !shared {
            return Err(Error::Forbidden("user profile is not visible".into()));
        }

        self.deps.users.get_profile(user_id).await
    }

    /// Loads an organization the caller belongs to.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the organization does not exist or the
    /// caller is not a member.
    pub async fn get_organization(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<Organization> {
        self.deps
            .organizations
            .list_for_user(caller_id)
            .await?
            .into_iter()
            .find(|o| o.id == organization_id)
            .ok_or_else(|| Error::NotFound("organization".into()))
    }

    /// Loads a channel visible to the caller.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the channel does not exist or is not
    /// visible, and [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn get_channel(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
    ) -> ruckchat_common::Result<Channel> {
        self.deps.channels.get_channel(caller_id, channel_id).await
    }

    /// Loads a DM conversation the caller participates in.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the conversation does not exist or the
    /// caller is not a member, and [`Error::Forbidden`] when the caller is not an
    /// organization member.
    pub async fn get_direct_message_conversation(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
    ) -> ruckchat_common::Result<DirectMessageConversation> {
        self.deps
            .direct_messages
            .get_conversation(caller_id, conversation_id)
            .await
    }

    /// Loads a single message visible to the caller.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message does not exist or is not
    /// visible, and [`Error::Forbidden`] when the caller cannot read the
    /// conversation.
    pub async fn get_message(
        &self,
        caller_id: UserId,
        message_id: MessageId,
    ) -> ruckchat_common::Result<Message> {
        self.deps.messages.get_message(caller_id, message_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{
        authorization::AuthorizationService, channel::ChannelServiceDeps,
        direct_message::DirectMessageServiceDeps, message::MessageServiceDeps,
        organization::OrganizationServiceDeps, user::UserServiceDeps,
    };
    use crate::testing::{
        MockChannelMembershipRepository, MockChannelRepository,
        MockDirectMessageConversationRepository, MockEventBus, MockMessageRepository,
        MockOrganizationMembershipRepository, MockOrganizationRepository,
        MockOrganizationSettingsRepository, MockUserRepository,
    };
    use ruckchat_domain::{
        Channel, ChannelMembership, ChannelMembershipRepository, ChannelRepository,
        ConversationType, OrganizationMembership, OrganizationMembershipRepository, Role, User,
        UserRepository,
    };
    use ruckchat_id::{ChannelId, OrganizationId};
    use std::sync::Arc;

    struct Harness {
        svc: McpService,
        memberships: Arc<MockOrganizationMembershipRepository>,
        channels_repo: Arc<MockChannelRepository>,
        channel_memberships: Arc<MockChannelMembershipRepository>,
        users_repo: Arc<MockUserRepository>,
    }

    fn service(require_confirmation: bool) -> Harness {
        let memberships = Arc::new(MockOrganizationMembershipRepository::new());
        let channels_repo = Arc::new(MockChannelRepository::new());
        let channel_memberships = Arc::new(MockChannelMembershipRepository::new());
        let messages_repo = Arc::new(MockMessageRepository::new());
        let conversations_repo = Arc::new(MockDirectMessageConversationRepository::new());
        let users_repo = Arc::new(MockUserRepository::new());
        let organizations_repo = Arc::new(MockOrganizationRepository::new());
        let settings_repo = Arc::new(MockOrganizationSettingsRepository::new());
        let authorization = AuthorizationService::new();
        let events = Arc::new(MockEventBus::new());

        let channels = ChannelService::new(ChannelServiceDeps {
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships.clone(),
            memberships: memberships.clone(),
            authorization: authorization.clone(),
        });

        let direct_messages = DirectMessageService::new(DirectMessageServiceDeps {
            conversations: conversations_repo.clone(),
            memberships: memberships.clone(),
        });

        let messages = MessageService::new(MessageServiceDeps {
            messages: messages_repo.clone(),
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships.clone(),
            memberships: memberships.clone(),
            conversations: conversations_repo.clone(),
            users: users_repo.clone(),
            authorization: authorization.clone(),
            events,
        });

        let users = UserService::new(UserServiceDeps {
            users: users_repo.clone(),
            memberships: memberships.clone(),
        });

        let organizations =
            crate::services::organization::OrganizationService::new(OrganizationServiceDeps {
                organizations: organizations_repo.clone(),
                users: users_repo.clone(),
                memberships: memberships.clone(),
                settings: settings_repo.clone(),
                authorization: authorization.clone(),
            });

        let svc = McpService::new(
            McpServiceDeps {
                channels,
                direct_messages,
                messages,
                users,
                organizations,
                memberships: memberships.clone(),
            },
            require_confirmation,
        );

        Harness {
            svc,
            memberships,
            channels_repo,
            channel_memberships,
            users_repo,
        }
    }

    async fn seed_org_with_channel(h: &Harness) -> (UserId, OrganizationId, ChannelId) {
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        h.users_repo.create(&user).await.unwrap();
        let org_id = OrganizationId::new();
        h.memberships
            .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let channel = Channel::new(org_id, "general", user.id, false).unwrap();
        h.channels_repo.create(&channel).await.unwrap();
        h.channel_memberships
            .create(&ChannelMembership::new(user.id, channel.id).unwrap())
            .await
            .unwrap();
        (user.id, org_id, channel.id)
    }

    #[tokio::test]
    async fn list_channels_requires_membership() {
        let h = service(true);
        let org_id = OrganizationId::new();
        let outsider = UserId::new();

        let err = h.svc.list_channels(outsider, org_id).await.unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn get_messages_requires_read_access() {
        let h = service(true);
        let (_user_id, _org_id, channel_id) = seed_org_with_channel(&h).await;
        let outsider = UserId::new();

        let err = h
            .svc
            .get_messages(
                outsider,
                channel_id.as_uuid(),
                ConversationType::Channel,
                Pagination::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn post_message_returns_confirmation_when_required() {
        let h = service(true);
        let (user_id, _org_id, channel_id) = seed_org_with_channel(&h).await;

        let result = h
            .svc
            .post_message(
                user_id,
                channel_id.as_uuid(),
                ConversationType::Channel,
                "hello".into(),
                None,
                false,
            )
            .await
            .unwrap();
        assert!(matches!(
            result,
            PostMessageResult::ConfirmationRequired { .. }
        ));
    }

    #[tokio::test]
    async fn post_message_posts_when_confirmed() {
        let h = service(true);
        let (user_id, _org_id, channel_id) = seed_org_with_channel(&h).await;

        let result = h
            .svc
            .post_message(
                user_id,
                channel_id.as_uuid(),
                ConversationType::Channel,
                "hello".into(),
                None,
                true,
            )
            .await
            .unwrap();
        assert!(matches!(result, PostMessageResult::Posted(_)));
    }

    #[tokio::test]
    async fn post_message_posts_when_confirmation_disabled() {
        let h = service(false);
        let (user_id, _org_id, channel_id) = seed_org_with_channel(&h).await;

        let result = h
            .svc
            .post_message(
                user_id,
                channel_id.as_uuid(),
                ConversationType::Channel,
                "hello".into(),
                None,
                false,
            )
            .await
            .unwrap();
        assert!(matches!(result, PostMessageResult::Posted(_)));
    }

    #[tokio::test]
    async fn search_messages_requires_membership() {
        let h = service(true);
        let org_id = OrganizationId::new();
        let outsider = UserId::new();

        let err = h
            .svc
            .search_messages(outsider, org_id, "hello", Pagination::default())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn empty_search_query_is_rejected() {
        let h = service(true);
        let (_user_id, org_id, _channel_id) = seed_org_with_channel(&h).await;

        let err = h
            .svc
            .search_messages(_user_id, org_id, "", Pagination::default())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn profile_visible_to_self() {
        let h = service(true);
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        h.users_repo.create(&user).await.unwrap();

        let profile = h.svc.get_user_profile(user.id, user.id).await.unwrap();
        assert_eq!(profile.email, "alice@example.com");
    }

    #[tokio::test]
    async fn profile_hidden_from_unrelated_user() {
        let h = service(true);
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        h.users_repo.create(&user).await.unwrap();
        let outsider = UserId::new();

        let err = h.svc.get_user_profile(outsider, user.id).await.unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
