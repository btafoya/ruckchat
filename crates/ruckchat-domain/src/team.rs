//! Team aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{OrganizationId, TeamId, UserId};
use serde::{Deserialize, Serialize};

/// A team of users within an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    /// Internal team identifier.
    pub id: TeamId,
    /// Organization this team belongs to.
    pub organization_id: OrganizationId,
    /// Unique team name within the organization.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// User who created the team.
    pub created_by: UserId,
    /// Timestamp when the team was created.
    pub created_at: OffsetDateTime,
    /// Timestamp of the last team update.
    pub updated_at: OffsetDateTime,
}

impl Team {
    /// Creates a team after validating the name.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the name is empty or too long.
    pub fn new(
        organization_id: OrganizationId,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        created_by: UserId,
    ) -> Result<Self> {
        let name = name.into();
        let description = description.map(Into::into);

        if name.is_empty() {
            return Err(Error::validation("team name must not be empty"));
        }
        if name.len() > 64 {
            return Err(Error::validation("team name must not exceed 64 characters"));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: TeamId::new(),
            organization_id,
            name,
            description,
            created_by,
            created_at: now,
            updated_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_team() {
        let team = Team::new(
            OrganizationId::new(),
            "Engineering",
            None::<String>,
            UserId::new(),
        )
        .expect("valid team");
        assert_eq!(team.name, "Engineering");
    }

    #[test]
    fn empty_team_name_rejected() {
        assert!(Team::new(OrganizationId::new(), "", None::<String>, UserId::new()).is_err());
    }
}
