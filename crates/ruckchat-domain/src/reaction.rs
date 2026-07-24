//! Reaction aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{MessageId, UserId};
use serde::{Deserialize, Serialize};

/// An emoji reaction to a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reaction {
    /// Message this reaction applies to.
    pub message_id: MessageId,
    /// User who added the reaction.
    pub user_id: UserId,
    /// Emoji character or shortcode.
    pub emoji: String,
    /// Timestamp when the reaction was added.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl Reaction {
    /// Creates a new reaction after validating the emoji is non-empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the emoji is empty.
    pub fn new(message_id: MessageId, user_id: UserId, emoji: impl Into<String>) -> Result<Self> {
        let emoji = emoji.into();
        if emoji.is_empty() {
            return Err(Error::validation("emoji must not be empty"));
        }

        Ok(Self {
            message_id,
            user_id,
            emoji,
            created_at: OffsetDateTime::now_utc(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_reaction() {
        let reaction =
            Reaction::new(MessageId::new(), UserId::new(), "👍").expect("valid reaction");
        assert_eq!(reaction.emoji, "👍");
    }

    #[test]
    fn empty_emoji_rejected() {
        assert!(Reaction::new(MessageId::new(), UserId::new(), "").is_err());
    }
}
