//! Organization membership aggregate.

use crate::role::Role;
use ruckchat_common::{Result, time::OffsetDateTime};
use ruckchat_id::{OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// Links a user to an organization with a role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationMembership {
    /// User identifier.
    pub user_id: UserId,
    /// Organization identifier.
    pub organization_id: OrganizationId,
    /// Membership role.
    pub role: Role,
    /// Timestamp when the user joined.
    pub joined_at: OffsetDateTime,
}

impl OrganizationMembership {
    /// Creates a new membership.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Validation`] if user or organization ids are invalid.
    /// Currently all UUIDs are accepted; this hook exists for future rules.
    pub fn new(user_id: UserId, organization_id: OrganizationId, role: Role) -> Result<Self> {
        Ok(Self {
            user_id,
            organization_id,
            role,
            joined_at: OffsetDateTime::now_utc(),
        })
    }

    /// Promotes or demotes the member.
    pub fn set_role(&mut self, role: Role) {
        self.role = role;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_membership() {
        let user_id = UserId::new();
        let org_id = OrganizationId::new();
        let membership =
            OrganizationMembership::new(user_id, org_id, Role::Member).expect("valid membership");
        assert_eq!(membership.user_id, user_id);
        assert_eq!(membership.organization_id, org_id);
        assert_eq!(membership.role, Role::Member);
    }

    #[test]
    fn change_role() {
        let mut membership =
            OrganizationMembership::new(UserId::new(), OrganizationId::new(), Role::Member)
                .expect("valid membership");
        membership.set_role(Role::Admin);
        assert_eq!(membership.role, Role::Admin);
    }
}
