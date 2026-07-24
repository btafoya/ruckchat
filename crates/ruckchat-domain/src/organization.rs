//! Organization aggregate.

use ruckchat_common::{
    Error, Result,
    time::OffsetDateTime,
    validation::{SLUG_MAX_LEN, SLUG_MIN_LEN, validate_slug},
};
use ruckchat_id::{OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// A tenant workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    /// Internal organization identifier.
    pub id: OrganizationId,
    /// Human-readable name.
    pub name: String,
    /// URL-safe unique slug.
    pub slug: String,
    /// User who owns the organization.
    pub owner_id: UserId,
    /// Timestamp when the organization was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Timestamp of the last update.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Organization {
    /// Creates a new organization after validating name and slug.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the name or slug is invalid.
    pub fn new(name: impl Into<String>, slug: impl Into<String>, owner_id: UserId) -> Result<Self> {
        let name = name.into();
        let slug = slug.into();

        if name.is_empty() {
            return Err(Error::validation("organization name must not be empty"));
        }
        let slug_len = slug.chars().count();
        if !(SLUG_MIN_LEN..=SLUG_MAX_LEN).contains(&slug_len) || !validate_slug(&slug) {
            return Err(Error::validation(format!(
                "organization slug must be {SLUG_MIN_LEN}-{SLUG_MAX_LEN} lowercase letters, numbers, and hyphens"
            )));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: OrganizationId::new(),
            name,
            slug,
            owner_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates the display name.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::validation("organization name must not be empty"));
        }
        self.name = name;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_organization() {
        let owner = UserId::new();
        let org = Organization::new("Acme", "acme", owner).expect("valid org");
        assert_eq!(org.name, "Acme");
        assert_eq!(org.slug, "acme");
        assert_eq!(org.owner_id, owner);
    }

    #[test]
    fn empty_name_rejected() {
        assert!(Organization::new("", "acme", UserId::new()).is_err());
    }

    #[test]
    fn invalid_slug_rejected() {
        assert!(Organization::new("Acme", "Acme", UserId::new()).is_err());
        assert!(Organization::new("Acme", "-acme", UserId::new()).is_err());
        assert!(Organization::new("Acme", "acme-", UserId::new()).is_err());
        assert!(Organization::new("Acme", "acme_corp", UserId::new()).is_err());
        assert!(Organization::new("Acme", "ab", UserId::new()).is_err());
    }

    #[test]
    fn update_name() {
        let mut org = Organization::new("Acme", "acme", UserId::new()).expect("valid org");
        org.set_name("Acme Inc").expect("update name");
        assert_eq!(org.name, "Acme Inc");
    }
}
