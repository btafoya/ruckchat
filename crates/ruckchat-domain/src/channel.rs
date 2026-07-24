//! Channel aggregate.

use ruckchat_common::{
    Error, Result,
    time::OffsetDateTime,
    validation::{CHANNEL_NAME_MAX_LEN, CHANNEL_NAME_MIN_LEN, validate_channel_name},
};
use ruckchat_id::{ChannelId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// A conversation space within an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    /// Internal channel identifier.
    pub id: ChannelId,
    /// Organization this channel belongs to.
    pub organization_id: OrganizationId,
    /// Unique channel name within the organization.
    pub name: String,
    /// Optional short description.
    pub topic: Option<String>,
    /// Optional longer explanation of the channel's purpose.
    pub purpose: Option<String>,
    /// Whether the channel is private.
    pub is_private: bool,
    /// User who created the channel.
    pub created_by: UserId,
    /// Timestamp when the channel was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Timestamp when the channel was archived, if applicable.
    #[serde(with = "time::serde::rfc3339::option")]
    pub archived_at: Option<OffsetDateTime>,
}

impl Channel {
    /// Creates a new channel after validating name and required associations.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the name is invalid.
    pub fn new(
        organization_id: OrganizationId,
        name: impl Into<String>,
        created_by: UserId,
        is_private: bool,
    ) -> Result<Self> {
        let name = name.into();
        if !validate_channel_name(&name) {
            return Err(Error::validation(format!(
                "channel name must be {CHANNEL_NAME_MIN_LEN}-{CHANNEL_NAME_MAX_LEN} lowercase letters, numbers, and hyphens"
            )));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: ChannelId::new(),
            organization_id,
            name,
            topic: None,
            purpose: None,
            is_private,
            created_by,
            created_at: now,
            archived_at: None,
        })
    }

    /// Sets the topic.
    pub fn set_topic(&mut self, topic: Option<impl Into<String>>) {
        self.topic = topic.map(Into::into);
    }

    /// Sets the purpose.
    pub fn set_purpose(&mut self, purpose: Option<impl Into<String>>) {
        self.purpose = purpose.map(Into::into);
    }

    /// Archives the channel.
    pub fn archive(&mut self) {
        if self.archived_at.is_none() {
            self.archived_at = Some(OffsetDateTime::now_utc());
        }
    }

    /// Restores an archived channel.
    pub fn unarchive(&mut self) {
        self.archived_at = None;
    }

    /// Returns true if the channel is archived.
    #[must_use]
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_channel() {
        let org_id = OrganizationId::new();
        let user_id = UserId::new();
        let channel = Channel::new(org_id, "general", user_id, false).expect("valid channel");
        assert_eq!(channel.name, "general");
        assert!(!channel.is_private);
        assert!(!channel.is_archived());
    }

    #[test]
    fn invalid_channel_name_rejected() {
        let org_id = OrganizationId::new();
        let user_id = UserId::new();
        assert!(Channel::new(org_id, "", user_id, false).is_err());
        assert!(Channel::new(org_id, "General", user_id, false).is_err());
        assert!(Channel::new(org_id, "-general", user_id, false).is_err());
        assert!(Channel::new(org_id, "general-", user_id, false).is_err());
        assert!(Channel::new(org_id, "general_chat", user_id, false).is_err());
    }

    #[test]
    fn archive_and_unarchive() {
        let mut channel = Channel::new(OrganizationId::new(), "general", UserId::new(), false)
            .expect("valid channel");
        channel.archive();
        assert!(channel.is_archived());
        channel.unarchive();
        assert!(!channel.is_archived());
    }
}
