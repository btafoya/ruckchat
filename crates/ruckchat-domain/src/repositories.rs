//! Repository interfaces for the RuckChat domain aggregates.
//!
//! These traits define the data-access boundary used by the service layer.
//! Concrete implementations using SQLx live in the server crate.

use crate::{
    Channel, ChannelMembership, CustomEmoji, DirectMessageConversation, File, Message,
    Organization, OrganizationMembership, OrganizationRole, OrganizationRolePermission,
    OrganizationSettings, Permission, Reaction, Role, Session, Team, TeamMembership, TeamRoom,
    User, WebPushSubscription,
};
use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_id::{
    ChannelId, CustomEmojiId, DirectMessageConversationId, FileId, MessageId, OrganizationId,
    OrganizationRoleId, PermissionId, SessionId, TeamId, UserId,
};
use uuid::Uuid;

/// User data access.
#[async_trait]
pub trait UserRepository {
    /// Persists a new user.
    async fn create(&self, user: &User) -> Result<()>;

    /// Loads a user by id.
    async fn by_id(&self, id: UserId) -> Result<Option<User>>;

    /// Loads a user by email address.
    async fn by_email(&self, email: &str) -> Result<Option<User>>;

    /// Updates an existing user.
    async fn update(&self, user: &User) -> Result<()>;
}

/// Session data access.
#[async_trait]
pub trait SessionRepository {
    /// Persists a new session.
    async fn create(&self, session: &Session) -> Result<()>;

    /// Loads a session by id.
    async fn by_id(&self, id: SessionId) -> Result<Option<Session>>;

    /// Loads a session by its token hash.
    async fn by_token_hash(&self, token_hash: &str) -> Result<Option<Session>>;

    /// Deletes all sessions that have expired.
    async fn delete_expired(&self) -> Result<u64>;

    /// Deletes a session by its token hash.
    async fn delete_by_token_hash(&self, token_hash: &str) -> Result<()>;
}

/// Organization data access.
#[async_trait]
pub trait OrganizationRepository {
    /// Persists a new organization.
    async fn create(&self, organization: &Organization) -> Result<()>;

    /// Loads an organization by id.
    async fn by_id(&self, id: OrganizationId) -> Result<Option<Organization>>;

    /// Loads an organization by slug.
    async fn by_slug(&self, slug: &str) -> Result<Option<Organization>>;

    /// Lists organizations a user belongs to.
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Organization>>;
}

/// Organization membership data access.
#[async_trait]
pub trait OrganizationMembershipRepository {
    /// Adds a user to an organization.
    async fn create(&self, membership: &OrganizationMembership) -> Result<()>;

    /// Loads a membership by composite key.
    async fn by_ids(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationMembership>>;

    /// Lists all members of an organization.
    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationMembership>>;

    /// Lists all organizations a user belongs to.
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<OrganizationMembership>>;

    /// Updates the member's role.
    async fn update_role(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        role: Role,
    ) -> Result<()>;

    /// Removes a user from an organization.
    async fn delete(&self, user_id: UserId, organization_id: OrganizationId) -> Result<()>;
}

/// Organization settings data access.
#[async_trait]
pub trait OrganizationSettingsRepository {
    /// Persists default settings for a new organization.
    async fn create(&self, settings: &OrganizationSettings) -> Result<()>;

    /// Loads settings for an organization.
    async fn by_organization_id(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationSettings>>;

    /// Updates quotas for an organization.
    async fn update(&self, settings: &OrganizationSettings) -> Result<()>;
}

/// Channel data access.
#[async_trait]
pub trait ChannelRepository {
    /// Persists a new channel.
    async fn create(&self, channel: &Channel) -> Result<()>;

    /// Updates an existing channel.
    async fn update(&self, channel: &Channel) -> Result<()>;

    /// Loads a channel by id.
    async fn by_id(&self, id: ChannelId) -> Result<Option<Channel>>;

    /// Lists channels in an organization.
    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<Channel>>;
}

/// Channel membership data access.
#[async_trait]
pub trait ChannelMembershipRepository {
    /// Adds a user to a channel.
    async fn create(&self, membership: &ChannelMembership) -> Result<()>;

    /// Loads a membership by composite key.
    async fn by_ids(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<Option<ChannelMembership>>;

    /// Lists members of a channel.
    async fn list_by_channel(&self, channel_id: ChannelId) -> Result<Vec<ChannelMembership>>;

    /// Lists channels a user is explicitly a member of.
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<ChannelMembership>>;

    /// Removes a user from a channel.
    async fn delete(&self, user_id: UserId, channel_id: ChannelId) -> Result<()>;
}

/// Direct message conversation data access.
#[async_trait]
pub trait DirectMessageConversationRepository {
    /// Persists a new DM conversation and its members.
    async fn create(&self, conversation: &DirectMessageConversation) -> Result<()>;

    /// Loads a conversation by id.
    async fn by_id(
        &self,
        id: DirectMessageConversationId,
    ) -> Result<Option<DirectMessageConversation>>;

    /// Lists DM conversations a user participates in within an organization.
    async fn list_by_user_and_organization(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<DirectMessageConversation>>;
}

/// Message data access.
#[async_trait]
pub trait MessageRepository {
    /// Persists a new message.
    async fn create(&self, message: &Message) -> Result<()>;

    /// Loads a message by id.
    async fn by_id(&self, id: MessageId) -> Result<Option<Message>>;

    /// Lists messages in a conversation, newest first.
    async fn list_by_conversation(
        &self,
        conversation_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>>;

    /// Updates an existing message.
    async fn update(&self, message: &Message) -> Result<()>;

    /// Searches visible message content using full-text search.
    async fn search(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>>;
}

/// Reaction data access.
#[async_trait]
pub trait ReactionRepository {
    /// Persists a new reaction.
    async fn create(&self, reaction: &Reaction) -> Result<()>;

    /// Lists reactions for a message.
    async fn list_by_message(&self, message_id: MessageId) -> Result<Vec<Reaction>>;

    /// Removes a reaction.
    async fn delete(&self, message_id: MessageId, user_id: UserId, emoji: &str) -> Result<()>;
}

/// File metadata data access.
#[async_trait]
pub trait FileRepository {
    /// Persists a new file record.
    async fn create(&self, file: &File) -> Result<()>;

    /// Loads a file by id.
    async fn by_id(&self, id: FileId) -> Result<Option<File>>;

    /// Lists files uploaded to an organization.
    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<File>>;

    /// Links a file to a message.
    async fn attach_to_message(&self, message_id: MessageId, file_id: FileId) -> Result<()>;
}

/// Custom organization role data access.
#[async_trait]
pub trait OrganizationRoleRepository {
    /// Persists a new role.
    async fn create(&self, role: &OrganizationRole) -> Result<()>;

    /// Loads a role by id.
    async fn by_id(&self, id: OrganizationRoleId) -> Result<Option<OrganizationRole>>;

    /// Lists roles in an organization.
    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationRole>>;
}

/// Permission data access.
#[async_trait]
pub trait PermissionRepository {
    /// Persists a new permission.
    async fn create(&self, permission: &Permission) -> Result<()>;

    /// Lists permissions in an organization.
    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Permission>>;
}

/// Role-permission grant data access.
#[async_trait]
pub trait RolePermissionRepository {
    /// Grants a permission to a role.
    async fn create(&self, grant: &OrganizationRolePermission) -> Result<()>;

    /// Lists permission ids granted to a role.
    async fn list_by_role(&self, role_id: OrganizationRoleId) -> Result<Vec<PermissionId>>;
}

/// Custom emoji data access.
#[async_trait]
pub trait CustomEmojiRepository {
    /// Persists a new emoji.
    async fn create(&self, emoji: &CustomEmoji) -> Result<()>;

    /// Loads an emoji by id.
    async fn by_id(&self, id: CustomEmojiId) -> Result<Option<CustomEmoji>>;

    /// Lists emoji in an organization.
    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomEmoji>>;
}

/// Team data access.
#[async_trait]
pub trait TeamRepository {
    /// Persists a new team.
    async fn create(&self, team: &Team) -> Result<()>;

    /// Loads a team by id.
    async fn by_id(&self, id: TeamId) -> Result<Option<Team>>;

    /// Lists teams in an organization.
    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<Team>>;
}

/// Team membership data access.
#[async_trait]
pub trait TeamMembershipRepository {
    /// Adds a user to a team.
    async fn create(&self, membership: &TeamMembership) -> Result<()>;

    /// Lists members of a team.
    async fn list_by_team(&self, team_id: TeamId) -> Result<Vec<TeamMembership>>;
}

/// Team-room link data access.
#[async_trait]
pub trait TeamRoomRepository {
    /// Links a channel to a team.
    async fn create(&self, link: &TeamRoom) -> Result<()>;

    /// Lists channels linked to a team.
    async fn list_by_team(&self, team_id: TeamId) -> Result<Vec<TeamRoom>>;
}

/// Web Push subscription data access.
#[async_trait]
pub trait WebPushSubscriptionRepository {
    /// Persists a subscription for a user, replacing an existing subscription for
    /// the same endpoint.
    async fn upsert(&self, subscription: &WebPushSubscription) -> Result<()>;

    /// Loads all subscriptions for a user.
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<WebPushSubscription>>;

    /// Deletes a subscription by user and endpoint.
    async fn delete_by_endpoint(&self, user_id: UserId, endpoint: &str) -> Result<()>;
}
