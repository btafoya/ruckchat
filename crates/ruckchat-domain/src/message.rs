//! Message aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime, validation::MESSAGE_CONTENT_MAX_LEN};
use ruckchat_id::{MessageId, UserId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Discriminator for the conversation a message belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationType {
    /// A channel conversation.
    Channel,
    /// A direct-message conversation.
    DirectMessage,
}

impl fmt::Display for ConversationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Channel => write!(f, "channel"),
            Self::DirectMessage => write!(f, "dm"),
        }
    }
}

/// Error returned when parsing a conversation type fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid conversation type: {0}")]
pub struct ParseConversationTypeError(String);

impl FromStr for ConversationType {
    type Err = ParseConversationTypeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "channel" => Ok(Self::Channel),
            "dm" => Ok(Self::DirectMessage),
            _ => Err(ParseConversationTypeError(s.into())),
        }
    }
}

/// A single communication unit in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Internal message identifier.
    pub id: MessageId,
    /// Conversation identifier (channel or DM UUID).
    pub conversation_id: Uuid,
    /// Discriminator for the conversation table.
    pub conversation_type: ConversationType,
    /// Optional parent message identifier for threads.
    pub parent_id: Option<MessageId>,
    /// User who authored the message.
    pub author_id: UserId,
    /// Message content.
    pub content: String,
    /// Timestamp when the message was created.
    pub created_at: OffsetDateTime,
    /// Timestamp of the last edit.
    pub updated_at: OffsetDateTime,
    /// Soft-delete timestamp.
    pub deleted_at: Option<OffsetDateTime>,
}

impl Message {
    /// Creates a new message after validating content length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when content is empty or too long.
    pub fn new(
        conversation_id: Uuid,
        conversation_type: ConversationType,
        author_id: UserId,
        content: impl Into<String>,
        parent_id: Option<MessageId>,
    ) -> Result<Self> {
        let content = content.into();
        let len = content.chars().count();
        if len == 0 {
            return Err(Error::validation("message content must not be empty"));
        }
        if len > MESSAGE_CONTENT_MAX_LEN {
            return Err(Error::validation(format!(
                "message content must not exceed {MESSAGE_CONTENT_MAX_LEN} characters"
            )));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: MessageId::new(),
            conversation_id,
            conversation_type,
            parent_id,
            author_id,
            content,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Edits the message content.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when new content is invalid.
    pub fn edit(&mut self, content: impl Into<String>) -> Result<()> {
        let content = content.into();
        let len = content.chars().count();
        if len == 0 {
            return Err(Error::validation("message content must not be empty"));
        }
        if len > MESSAGE_CONTENT_MAX_LEN {
            return Err(Error::validation(format!(
                "message content must not exceed {MESSAGE_CONTENT_MAX_LEN} characters"
            )));
        }
        self.content = content;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }

    /// Soft-deletes the message.
    pub fn delete(&mut self) {
        if self.deleted_at.is_none() {
            self.deleted_at = Some(OffsetDateTime::now_utc());
            self.content.clear();
        }
    }

    /// Returns true if the message is deleted.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_message() {
        let msg = Message::new(
            Uuid::new_v4(),
            ConversationType::Channel,
            UserId::new(),
            "hello",
            None,
        )
        .expect("valid message");
        assert_eq!(msg.content, "hello");
        assert!(!msg.is_deleted());
    }

    #[test]
    fn empty_content_rejected() {
        assert!(
            Message::new(
                Uuid::new_v4(),
                ConversationType::Channel,
                UserId::new(),
                "",
                None
            )
            .is_err()
        );
    }

    #[test]
    fn oversized_content_rejected() {
        assert!(
            Message::new(
                Uuid::new_v4(),
                ConversationType::Channel,
                UserId::new(),
                "x".repeat(MESSAGE_CONTENT_MAX_LEN + 1),
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn edit_message() {
        let mut msg = Message::new(
            Uuid::new_v4(),
            ConversationType::Channel,
            UserId::new(),
            "hello",
            None,
        )
        .expect("valid message");
        msg.edit("hello world").expect("edit message");
        assert_eq!(msg.content, "hello world");
    }

    #[test]
    fn delete_message() {
        let mut msg = Message::new(
            Uuid::new_v4(),
            ConversationType::Channel,
            UserId::new(),
            "hello",
            None,
        )
        .expect("valid message");
        msg.delete();
        assert!(msg.is_deleted());
        assert!(msg.content.is_empty());
    }

    #[test]
    fn conversation_type_round_trip() {
        for ct in [ConversationType::Channel, ConversationType::DirectMessage] {
            let text = ct.to_string();
            let parsed = ConversationType::from_str(&text).expect("parse conversation type");
            assert_eq!(parsed, ct);
        }
    }
}
