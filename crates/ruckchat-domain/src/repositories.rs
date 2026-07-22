//! Repository interfaces for the RuckChat domain aggregates.
//!
//! These traits define the data-access boundary used by the service layer.
//! Concrete implementations using SQLx live in the server crate.

use crate::{
    Channel, ChannelMembership, DirectMessageConversation, File, Message, Organization,
    OrganizationMembership, OrganizationSettings, Reaction, Role, Session, User,
};
use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_id::{
    ChannelId, DirectMessageConversationId, FileId, MessageId, OrganizationId, SessionId, UserId,
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
}
