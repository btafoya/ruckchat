//! Organization membership role value object.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Role of a user within an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Organization owner. Has full control and cannot be removed by others.
    #[default]
    Owner,
    /// Organization administrator. Can manage channels and members but not
    /// transfer ownership.
    Admin,
    /// Regular organization member.
    Member,
}

impl Role {
    /// Returns true if this role can manage organization-wide settings and
    /// members.
    #[must_use]
    pub fn is_manager(self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Returns true if this role can delete or edit any content regardless of
    /// authorship.
    #[must_use]
    pub fn is_moderator(self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owner => write!(f, "owner"),
            Self::Admin => write!(f, "admin"),
            Self::Member => write!(f, "member"),
        }
    }
}

/// Error returned when parsing a role from a string fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid role: {0}")]
pub struct ParseRoleError(String);

impl FromStr for Role {
    type Err = ParseRoleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            _ => Err(ParseRoleError(s.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_default_is_member_or_owner() {
        // Default is owner because the schema default/creation path typically
        // starts with an owner. Revisit if the default should change.
        let role: Role = Default::default();
        assert_eq!(role, Role::Owner);
    }

    #[test]
    fn managers_and_moderators() {
        assert!(Role::Owner.is_manager());
        assert!(Role::Admin.is_manager());
        assert!(!Role::Member.is_manager());
        assert!(Role::Owner.is_moderator());
        assert!(Role::Admin.is_moderator());
        assert!(!Role::Member.is_moderator());
    }

    #[test]
    fn role_round_trip() {
        for role in [Role::Owner, Role::Admin, Role::Member] {
            let text = role.to_string();
            let parsed = Role::from_str(&text).expect("parse role");
            assert_eq!(parsed, role);
        }
    }

    #[test]
    fn invalid_role_fails() {
        assert!(Role::from_str("superuser").is_err());
    }
}
