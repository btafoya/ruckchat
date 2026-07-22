//! WebSocket event types and publishing trait.
//!
//! Services emit domain events through [`EventBus`] without knowing whether the
//! transport is WebSocket, a plugin hook, or a no-op test recorder.

use ruckchat_domain::ConversationType;
use ruckchat_id::{MessageId, UserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Publishes real-time events to interested clients.
#[async_trait::async_trait]
pub trait EventBus: Send + Sync {
    /// Broadcasts a newly created message to conversation members.
    async fn publish_message_created(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()>;

    /// Broadcasts an edited message to conversation members.
    async fn publish_message_updated(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()>;

    /// Broadcasts a deleted message to conversation members.
    async fn publish_message_deleted(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()>;

    /// Broadcasts a new reaction to conversation members.
    async fn publish_reaction_added(
        &self,
        reaction: &ruckchat_domain::Reaction,
    ) -> ruckchat_common::Result<()>;

    /// Broadcasts a removed reaction to conversation members.
    async fn publish_reaction_removed(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> ruckchat_common::Result<()>;

    /// Broadcasts a typing indicator to conversation members.
    async fn publish_typing(
        &self,
        user_id: UserId,
        conversation_id: Uuid,
        conversation_type: ConversationType,
    ) -> ruckchat_common::Result<()>;

    /// Broadcasts a presence change for a user.
    async fn publish_presence(
        &self,
        user_id: UserId,
        status: PresenceStatus,
    ) -> ruckchat_common::Result<()>;
}

/// A server-to-client event payload.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    /// A message was posted.
    MessageCreated {
        /// The new message.
        message: ruckchat_domain::Message,
    },
    /// A message was edited.
    MessageUpdated {
        /// The updated message.
        message: ruckchat_domain::Message,
    },
    /// A message was soft-deleted.
    MessageDeleted {
        /// The deleted message.
        message: ruckchat_domain::Message,
    },
    /// A reaction was added.
    ReactionAdded {
        /// The new reaction.
        reaction: ruckchat_domain::Reaction,
    },
    /// A reaction was removed.
    ReactionRemoved {
        /// Message the reaction belonged to.
        message_id: MessageId,
        /// User who removed the reaction.
        user_id: UserId,
        /// Emoji that was removed.
        emoji: String,
    },
    /// A user is typing in a conversation.
    Typing {
        /// User who is typing.
        user_id: UserId,
        /// Conversation being typed in.
        conversation_id: Uuid,
        /// Kind of conversation.
        conversation_type: ConversationType,
    },
    /// A user's online/offline status changed.
    Presence {
        /// User whose presence changed.
        user_id: UserId,
        /// New presence status.
        status: PresenceStatus,
    },
    /// Connection is ready to receive events.
    ConnectionEstablished {
        /// Authenticated user id.
        user_id: UserId,
    },
    /// A client-sent message could not be processed.
    Error {
        /// Error details.
        error: ErrorEvent,
    },
}

impl ServerEvent {
    /// Returns the canonical event name used in the outer envelope.
    #[must_use]
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::MessageCreated { .. } => "message.created",
            Self::MessageUpdated { .. } => "message.updated",
            Self::MessageDeleted { .. } => "message.deleted",
            Self::ReactionAdded { .. } => "reaction.added",
            Self::ReactionRemoved { .. } => "reaction.removed",
            Self::Typing { .. } => "typing.updated",
            Self::Presence { .. } => "presence.updated",
            Self::ConnectionEstablished { .. } => "connection.established",
            Self::Error { .. } => "error",
        }
    }
}

/// Uniform server-to-client envelope.
#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope {
    /// Event name matching the payload variant.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Unique event identifier.
    pub id: Uuid,
    /// UTC timestamp when the event was emitted.
    pub timestamp: OffsetDateTime,
    /// Event payload.
    pub payload: ServerEvent,
}

impl EventEnvelope {
    /// Wraps a server event in an envelope with a fresh id and timestamp.
    #[must_use]
    pub fn new(payload: ServerEvent) -> Self {
        Self {
            event_type: payload.event_type().into(),
            id: Uuid::new_v4(),
            timestamp: OffsetDateTime::now_utc(),
            payload,
        }
    }
}

/// Client-to-server WebSocket message.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Start receiving events for an organization.
    SubscribeOrganization {
        /// Organization to subscribe to.
        organization_id: ruckchat_id::OrganizationId,
    },
    /// Stop receiving events for an organization.
    UnsubscribeOrganization {
        /// Organization to unsubscribe from.
        organization_id: ruckchat_id::OrganizationId,
    },
    /// Notify conversation members that the caller is typing.
    Typing {
        /// Conversation being typed in.
        conversation_id: Uuid,
        /// Kind of conversation.
        conversation_type: ConversationType,
    },
    /// Client heartbeat.
    Ping,
}

/// Presence status of a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    /// User has at least one open connection.
    Online,
    /// User has no open connections.
    Offline,
}

/// Error response sent to the client for invalid WebSocket messages.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorEvent {
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}
