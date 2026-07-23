//! Team membership aggregate.

use ruckchat_common::time::OffsetDateTime;
use ruckchat_id::{TeamId, UserId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Role a user has within a team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    /// Team owner. Has full control over the team.
    #[default]
    Owner,
    /// Team leader. Can manage team members and rooms.
    Leader,
    /// Regular team member.
    Member,
}

impl TeamRole {
    /// Returns true if this role can manage team membership and rooms.
    #[must_use]
    pub fn is_manager(self) -> bool {
        matches!(self, Self::Owner | Self::Leader)
    }
}

impl fmt::Display for TeamRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owner => write!(f, "owner"),
            Self::Leader => write!(f, "leader"),
            Self::Member => write!(f, "member"),
        }
    }
}

/// Error returned when parsing a team role from a string fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid team role: {0}")]
pub struct ParseTeamRoleError(String);

impl FromStr for TeamRole {
    type Err = ParseTeamRoleError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::Owner),
            "leader" => Ok(Self::Leader),
            "member" => Ok(Self::Member),
            _ => Err(ParseTeamRoleError(s.into())),
        }
    }
}

/// A user's membership in a team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamMembership {
    /// Team identifier.
    pub team_id: TeamId,
    /// User identifier.
    pub user_id: UserId,
    /// Role within the team.
    pub role: TeamRole,
    /// Timestamp when the user joined the team.
    pub joined_at: OffsetDateTime,
}

impl TeamMembership {
    /// Creates a team membership.
    #[must_use]
    pub fn new(team_id: TeamId, user_id: UserId, role: TeamRole) -> Self {
        Self {
            team_id,
            user_id,
            role,
            joined_at: OffsetDateTime::now_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_role_round_trip() {
        for role in [TeamRole::Owner, TeamRole::Leader, TeamRole::Member] {
            let text = role.to_string();
            let parsed = TeamRole::from_str(&text).expect("parse role");
            assert_eq!(parsed, role);
        }
    }

    #[test]
    fn create_membership() {
        let membership = TeamMembership::new(TeamId::new(), UserId::new(), TeamRole::Member);
        assert_eq!(membership.role, TeamRole::Member);
    }
}
