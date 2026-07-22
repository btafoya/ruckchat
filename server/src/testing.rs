//! In-memory mock repositories for unit tests.
//!
//! These mocks implement the domain repository traits with simple storage. They
//! are not thread-safe for concurrent mutation, but they are sufficient for
//! single-threaded async tests using `tokio::test` with `current_thread`.

use crate::services::events::{EventBus, PresenceStatus, ServerEvent};
use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{
    Channel, ChannelMembership, ChannelMembershipRepository, ChannelRepository,
    DirectMessageConversation, DirectMessageConversationRepository, File, FileRepository, Message,
    MessageRepository, Organization, OrganizationMembership, OrganizationMembershipRepository,
    OrganizationRepository, OrganizationSettings, OrganizationSettingsRepository, Reaction,
    ReactionRepository, Role, Session, SessionRepository, User, UserRepository,
};
use ruckchat_id::{
    ChannelId, DirectMessageConversationId, FileId, MessageId, OrganizationId, SessionId, UserId,
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// In-memory user repository.
#[derive(Debug, Default, Clone)]
pub struct MockUserRepository {
    users: Arc<Mutex<Vec<User>>>,
}

impl MockUserRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create(&self, user: &User) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        if users.iter().any(|u| u.email == user.email) {
            return Err(ruckchat_common::Error::Conflict(
                "email already exists".into(),
            ));
        }
        users.push(user.clone());
        Ok(())
    }

    async fn by_id(&self, id: UserId) -> Result<Option<User>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == id)
            .cloned())
    }

    async fn by_email(&self, email: &str) -> Result<Option<User>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.email == email)
            .cloned())
    }

    async fn update(&self, user: &User) -> Result<()> {
        let mut users = self.users.lock().unwrap();
        let existing_id = users
            .iter()
            .position(|u| u.id == user.id)
            .ok_or_else(|| ruckchat_common::Error::NotFound("user".into()))?;
        if users
            .iter()
            .any(|u| u.email == user.email && u.id != user.id)
        {
            return Err(ruckchat_common::Error::Conflict(
                "email already exists".into(),
            ));
        }
        users[existing_id] = user.clone();
        Ok(())
    }
}

/// In-memory session repository.
#[derive(Debug, Default, Clone)]
pub struct MockSessionRepository {
    sessions: Arc<Mutex<Vec<Session>>>,
}

impl MockSessionRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SessionRepository for MockSessionRepository {
    async fn create(&self, session: &Session) -> Result<()> {
        self.sessions.lock().unwrap().push(session.clone());
        Ok(())
    }

    async fn by_id(&self, id: SessionId) -> Result<Option<Session>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == id)
            .cloned())
    }

    async fn by_token_hash(&self, token_hash: &str) -> Result<Option<Session>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.token_hash == token_hash)
            .cloned())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let mut sessions = self.sessions.lock().unwrap();
        let before = sessions.len();
        sessions.retain(|s| !s.is_expired());
        Ok((before - sessions.len()) as u64)
    }

    async fn delete_by_token_hash(&self, token_hash: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let before = sessions.len();
        sessions.retain(|s| s.token_hash != token_hash);
        if sessions.len() == before {
            return Err(ruckchat_common::Error::NotFound("session".into()));
        }
        Ok(())
    }
}

/// In-memory organization repository.
#[derive(Debug, Default, Clone)]
pub struct MockOrganizationRepository {
    organizations: Arc<Mutex<Vec<Organization>>>,
}

impl MockOrganizationRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl OrganizationRepository for MockOrganizationRepository {
    async fn create(&self, organization: &Organization) -> Result<()> {
        let mut orgs = self.organizations.lock().unwrap();
        if orgs.iter().any(|o| o.slug == organization.slug) {
            return Err(ruckchat_common::Error::Conflict(
                "slug already exists".into(),
            ));
        }
        orgs.push(organization.clone());
        Ok(())
    }

    async fn by_id(&self, id: OrganizationId) -> Result<Option<Organization>> {
        Ok(self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.id == id)
            .cloned())
    }

    async fn by_slug(&self, slug: &str) -> Result<Option<Organization>> {
        Ok(self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.slug == slug)
            .cloned())
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Organization>> {
        let ids: Vec<Uuid> = {
            let orgs = self.organizations.lock().unwrap();
            orgs.iter()
                .filter(|o| o.owner_id == user_id)
                .map(|o| o.id.as_uuid())
                .collect()
        };
        Ok(ids
            .into_iter()
            .filter_map(|id| {
                self.organizations
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|o| o.id.as_uuid() == id)
                    .cloned()
            })
            .collect())
    }
}

/// In-memory organization membership repository.
#[derive(Debug, Default, Clone)]
pub struct MockOrganizationMembershipRepository {
    memberships: Arc<Mutex<Vec<OrganizationMembership>>>,
}

impl MockOrganizationMembershipRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl OrganizationMembershipRepository for MockOrganizationMembershipRepository {
    async fn create(&self, membership: &OrganizationMembership) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        if let Some(existing) = memberships.iter_mut().find(|m| {
            m.user_id == membership.user_id && m.organization_id == membership.organization_id
        }) {
            existing.role = membership.role;
            return Ok(());
        }
        memberships.push(membership.clone());
        Ok(())
    }

    async fn by_ids(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationMembership>> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.user_id == user_id && m.organization_id == organization_id)
            .cloned())
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationMembership>> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<OrganizationMembership>> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn update_role(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        role: Role,
    ) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        let membership = memberships
            .iter_mut()
            .find(|m| m.user_id == user_id && m.organization_id == organization_id)
            .ok_or_else(|| ruckchat_common::Error::NotFound("membership".into()))?;
        membership.role = role;
        Ok(())
    }

    async fn delete(&self, user_id: UserId, organization_id: OrganizationId) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        let idx = memberships
            .iter()
            .position(|m| m.user_id == user_id && m.organization_id == organization_id)
            .ok_or_else(|| ruckchat_common::Error::NotFound("membership".into()))?;
        memberships.remove(idx);
        Ok(())
    }
}

/// In-memory organization settings repository.
#[derive(Debug, Default, Clone)]
pub struct MockOrganizationSettingsRepository {
    settings: Arc<Mutex<Vec<OrganizationSettings>>>,
}

impl MockOrganizationSettingsRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl OrganizationSettingsRepository for MockOrganizationSettingsRepository {
    async fn create(&self, settings: &OrganizationSettings) -> Result<()> {
        let mut list = self.settings.lock().unwrap();
        if let Some(existing) = list
            .iter_mut()
            .find(|s| s.organization_id == settings.organization_id)
        {
            *existing = settings.clone();
            return Ok(());
        }
        list.push(settings.clone());
        Ok(())
    }

    async fn by_organization_id(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationSettings>> {
        Ok(self
            .settings
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.organization_id == organization_id)
            .cloned())
    }

    async fn update(&self, settings: &OrganizationSettings) -> Result<()> {
        self.create(settings).await
    }
}

/// In-memory channel repository.
#[derive(Debug, Default, Clone)]
pub struct MockChannelRepository {
    channels: Arc<Mutex<Vec<Channel>>>,
}

impl MockChannelRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ChannelRepository for MockChannelRepository {
    async fn create(&self, channel: &Channel) -> Result<()> {
        let mut channels = self.channels.lock().unwrap();
        if channels
            .iter()
            .any(|c| c.organization_id == channel.organization_id && c.name == channel.name)
        {
            return Err(ruckchat_common::Error::Conflict(
                "channel name already exists".into(),
            ));
        }
        channels.push(channel.clone());
        Ok(())
    }

    async fn update(&self, channel: &Channel) -> Result<()> {
        let mut channels = self.channels.lock().unwrap();
        let existing = channels
            .iter_mut()
            .find(|c| c.id == channel.id)
            .ok_or_else(|| ruckchat_common::Error::NotFound("channel".into()))?;
        *existing = channel.clone();
        Ok(())
    }

    async fn by_id(&self, id: ChannelId) -> Result<Option<Channel>> {
        Ok(self
            .channels
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }

    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<Channel>> {
        Ok(self
            .channels
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.organization_id == organization_id)
            .cloned()
            .collect())
    }
}

/// In-memory channel membership repository.
#[derive(Debug, Default, Clone)]
pub struct MockChannelMembershipRepository {
    memberships: Arc<Mutex<Vec<ChannelMembership>>>,
}

impl MockChannelMembershipRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ChannelMembershipRepository for MockChannelMembershipRepository {
    async fn create(&self, membership: &ChannelMembership) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        if memberships
            .iter()
            .any(|m| m.user_id == membership.user_id && m.channel_id == membership.channel_id)
        {
            return Err(ruckchat_common::Error::Conflict(
                "already a channel member".into(),
            ));
        }
        memberships.push(membership.clone());
        Ok(())
    }

    async fn by_ids(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<Option<ChannelMembership>> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.user_id == user_id && m.channel_id == channel_id)
            .cloned())
    }

    async fn list_by_channel(&self, channel_id: ChannelId) -> Result<Vec<ChannelMembership>> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.channel_id == channel_id)
            .cloned()
            .collect())
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<ChannelMembership>> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, user_id: UserId, channel_id: ChannelId) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        let idx = memberships
            .iter()
            .position(|m| m.user_id == user_id && m.channel_id == channel_id)
            .ok_or_else(|| ruckchat_common::Error::NotFound("channel membership".into()))?;
        memberships.remove(idx);
        Ok(())
    }
}

/// In-memory direct message conversation repository.
#[derive(Debug, Default, Clone)]
pub struct MockDirectMessageConversationRepository {
    conversations: Arc<Mutex<Vec<DirectMessageConversation>>>,
}

impl MockDirectMessageConversationRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DirectMessageConversationRepository for MockDirectMessageConversationRepository {
    async fn create(&self, conversation: &DirectMessageConversation) -> Result<()> {
        self.conversations
            .lock()
            .unwrap()
            .push(conversation.clone());
        Ok(())
    }

    async fn by_id(
        &self,
        id: DirectMessageConversationId,
    ) -> Result<Option<DirectMessageConversation>> {
        Ok(self
            .conversations
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }

    async fn list_by_user_and_organization(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<DirectMessageConversation>> {
        Ok(self
            .conversations
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.organization_id == organization_id && c.member_ids.contains(&user_id))
            .cloned()
            .collect())
    }
}

/// In-memory message repository.
#[derive(Debug, Default, Clone)]
pub struct MockMessageRepository {
    messages: Arc<Mutex<Vec<Message>>>,
}

impl MockMessageRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MessageRepository for MockMessageRepository {
    async fn create(&self, message: &Message) -> Result<()> {
        let mut messages = self.messages.lock().unwrap();
        if let Some(existing) = messages.iter_mut().find(|m| m.id == message.id) {
            *existing = message.clone();
            return Ok(());
        }
        messages.push(message.clone());
        Ok(())
    }

    async fn by_id(&self, id: MessageId) -> Result<Option<Message>> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.id == id)
            .cloned())
    }

    async fn list_by_conversation(
        &self,
        conversation_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let messages = self.messages.lock().unwrap();
        let mut filtered: Vec<Message> = messages
            .iter()
            .filter(|m| m.conversation_id == conversation_id && !m.is_deleted())
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(filtered.len());
        Ok(filtered[start..end].to_vec())
    }

    async fn update(&self, message: &Message) -> Result<()> {
        let mut messages = self.messages.lock().unwrap();
        let idx = messages
            .iter()
            .position(|m| m.id == message.id)
            .ok_or_else(|| ruckchat_common::Error::NotFound("message".into()))?;
        messages[idx] = message.clone();
        Ok(())
    }

    async fn search(
        &self,
        _caller_id: UserId,
        _organization_id: OrganizationId,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let messages = self.messages.lock().unwrap();
        let lower_query = query.to_lowercase();
        let mut filtered: Vec<Message> = messages
            .iter()
            .filter(|m| !m.is_deleted() && m.content.to_lowercase().contains(&lower_query))
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(filtered.len());
        Ok(filtered[start..end].to_vec())
    }
}

/// In-memory reaction repository.
#[derive(Debug, Default, Clone)]
pub struct MockReactionRepository {
    reactions: Arc<Mutex<Vec<Reaction>>>,
}

impl MockReactionRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ReactionRepository for MockReactionRepository {
    async fn create(&self, reaction: &Reaction) -> Result<()> {
        self.reactions.lock().unwrap().push(reaction.clone());
        Ok(())
    }

    async fn list_by_message(&self, message_id: MessageId) -> Result<Vec<Reaction>> {
        Ok(self
            .reactions
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.message_id == message_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, message_id: MessageId, user_id: UserId, emoji: &str) -> Result<()> {
        let mut reactions = self.reactions.lock().unwrap();
        let idx = reactions
            .iter()
            .position(|r| r.message_id == message_id && r.user_id == user_id && r.emoji == emoji)
            .ok_or_else(|| ruckchat_common::Error::NotFound("reaction".into()))?;
        reactions.remove(idx);
        Ok(())
    }
}

/// In-memory event bus that records published events for tests.
#[derive(Debug, Default, Clone)]
pub struct MockEventBus {
    events: Arc<Mutex<Vec<ServerEvent>>>,
}

impl MockEventBus {
    /// Creates an empty event bus.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all events published so far.
    #[must_use]
    pub fn events(&self) -> Vec<ServerEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Clears recorded events.
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

#[async_trait]
impl EventBus for MockEventBus {
    async fn publish_message_created(&self, message: &Message) -> ruckchat_common::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(ServerEvent::MessageCreated {
                message: message.clone(),
            });
        Ok(())
    }

    async fn publish_message_updated(&self, message: &Message) -> ruckchat_common::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(ServerEvent::MessageUpdated {
                message: message.clone(),
            });
        Ok(())
    }

    async fn publish_message_deleted(&self, message: &Message) -> ruckchat_common::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(ServerEvent::MessageDeleted {
                message: message.clone(),
            });
        Ok(())
    }

    async fn publish_reaction_added(&self, reaction: &Reaction) -> ruckchat_common::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(ServerEvent::ReactionAdded {
                reaction: reaction.clone(),
            });
        Ok(())
    }

    async fn publish_reaction_removed(
        &self,
        message_id: ruckchat_id::MessageId,
        user_id: ruckchat_id::UserId,
        emoji: &str,
    ) -> ruckchat_common::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(ServerEvent::ReactionRemoved {
                message_id,
                user_id,
                emoji: emoji.into(),
            });
        Ok(())
    }

    async fn publish_typing(
        &self,
        user_id: ruckchat_id::UserId,
        conversation_id: uuid::Uuid,
        conversation_type: ruckchat_domain::ConversationType,
    ) -> ruckchat_common::Result<()> {
        self.events.lock().unwrap().push(ServerEvent::Typing {
            user_id,
            conversation_id,
            conversation_type,
        });
        Ok(())
    }

    async fn publish_presence(
        &self,
        user_id: ruckchat_id::UserId,
        status: PresenceStatus,
    ) -> ruckchat_common::Result<()> {
        self.events
            .lock()
            .unwrap()
            .push(ServerEvent::Presence { user_id, status });
        Ok(())
    }
}

/// In-memory file repository.
#[derive(Debug, Default, Clone)]
pub struct MockFileRepository {
    files: Arc<Mutex<Vec<File>>>,
    attachments: Arc<Mutex<Vec<(MessageId, FileId)>>>,
}

impl MockFileRepository {
    /// Creates an empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl FileRepository for MockFileRepository {
    async fn create(&self, file: &File) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        if let Some(existing) = files.iter_mut().find(|f| f.id == file.id) {
            *existing = file.clone();
            return Ok(());
        }
        files.push(file.clone());
        Ok(())
    }

    async fn by_id(&self, id: FileId) -> Result<Option<File>> {
        Ok(self
            .files
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.id == id)
            .cloned())
    }

    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<File>> {
        Ok(self
            .files
            .lock()
            .unwrap()
            .iter()
            .filter(|f| f.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn attach_to_message(&self, message_id: MessageId, file_id: FileId) -> Result<()> {
        self.attachments.lock().unwrap().push((message_id, file_id));
        Ok(())
    }
}
