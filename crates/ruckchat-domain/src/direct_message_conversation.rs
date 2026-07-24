//! Direct message conversation aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{DirectMessageConversationId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// A conversation between two or more users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectMessageConversation {
    /// Internal conversation identifier.
    pub id: DirectMessageConversationId,
    /// Organization this DM belongs to.
    pub organization_id: OrganizationId,
    /// Users participating in the conversation.
    pub member_ids: Vec<UserId>,
    /// Timestamp when the conversation was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl DirectMessageConversation {
    /// Creates a direct message conversation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if there are fewer than two unique members
    /// or more than a reasonable limit.
    pub fn new(
        organization_id: OrganizationId,
        member_ids: impl IntoIterator<Item = UserId>,
    ) -> Result<Self> {
        let mut members: Vec<UserId> = member_ids.into_iter().collect();
        members.sort_unstable();
        members.dedup();

        if members.len() < 2 {
            return Err(Error::validation(
                "direct message conversation requires at least two members",
            ));
        }
        if members.len() > 20 {
            return Err(Error::validation(
                "direct message conversation cannot exceed 20 members",
            ));
        }

        Ok(Self {
            id: DirectMessageConversationId::new(),
            organization_id,
            member_ids: members,
            created_at: OffsetDateTime::now_utc(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_conversation() {
        let a = UserId::new();
        let b = UserId::new();
        let conv = DirectMessageConversation::new(OrganizationId::new(), [a, b]).expect("valid dm");
        assert_eq!(conv.member_ids.len(), 2);
    }

    #[test]
    fn duplicate_members_are_deduplicated_then_rejected() {
        let a = UserId::new();
        assert!(DirectMessageConversation::new(OrganizationId::new(), [a, a]).is_err());
    }

    #[test]
    fn single_member_rejected() {
        assert!(DirectMessageConversation::new(OrganizationId::new(), [UserId::new()]).is_err());
    }
}
