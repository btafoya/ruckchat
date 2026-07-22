//! Strongly typed identifiers for RuckChat domain entities.
//!
//! Each identifier is a newtype wrapper around a [`Uuid`], giving compile-time
//! guarantees that an API receiving a [`UserId`] is not passed a [`ChannelId`]
//! by accident.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Generates a strongly typed UUID wrapper and a constructor for new random ids.
macro_rules! id_type {
    ($name:ident) => {
        /// Strongly typed UUID identifier.
        #[derive(
            Debug,
            Default,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new random identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Constructs an identifier from the given raw UUID.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the underlying raw UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            /// Parses an identifier from a UUID string.
            ///
            /// # Errors
            ///
            /// Returns [`IdParseError`] when the string is not a valid UUID.
            pub fn parse_str(s: &str) -> Result<Self, IdParseError> {
                Ok(Self(Uuid::parse_str(s).map_err(IdParseError)?))
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }
    };
}

/// Error returned when parsing an identifier from a string fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid identifier: {0}")]
pub struct IdParseError(#[from] uuid::Error);

id_type!(UserId);
id_type!(OrganizationId);
id_type!(ChannelId);
id_type!(DirectMessageConversationId);
id_type!(MessageId);
id_type!(FileId);
id_type!(SessionId);

/// A polymorphic conversation identifier: either a channel or a DM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum ConversationId {
    /// A channel conversation.
    Channel(ChannelId),
    /// A direct-message conversation.
    DirectMessage(DirectMessageConversationId),
}

impl ConversationId {
    /// Returns the raw UUID backing the identifier.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        match self {
            Self::Channel(id) => id.as_uuid(),
            Self::DirectMessage(id) => id.as_uuid(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_unique() {
        let a = UserId::new();
        let b = UserId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn parse_round_trip() {
        let id = UserId::new();
        let text = id.to_string();
        let parsed = UserId::parse_str(&text).expect("parse uuid string");
        assert_eq!(parsed, id);
    }

    #[test]
    fn parse_invalid_uuid_fails() {
        assert!(UserId::parse_str("not-a-uuid").is_err());
    }

    #[test]
    fn serde_round_trip() {
        let id = ChannelId::new();
        let json = serde_json::to_string(&id).expect("serialize");
        let parsed: ChannelId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, id);
    }

    #[test]
    fn conversation_id_serde() {
        let id = ConversationId::Channel(ChannelId::new());
        let json = serde_json::to_string(&id).expect("serialize");
        let parsed: ConversationId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, id);
    }
}
