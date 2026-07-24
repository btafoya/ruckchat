//! Custom organization role aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{OrganizationId, OrganizationRoleId};
use serde::{Deserialize, Serialize};

/// A custom role within an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationRole {
    /// Internal role identifier.
    pub id: OrganizationRoleId,
    /// Organization this role belongs to.
    pub organization_id: OrganizationId,
    /// Unique role name within the organization.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Timestamp when the role was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Timestamp of the last role update.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl OrganizationRole {
    /// Creates a new organization role after validating the name.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the name is empty or too long.
    pub fn new(
        organization_id: OrganizationId,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let name = name.into();
        let description = description.map(Into::into);

        if name.is_empty() {
            return Err(Error::validation("role name must not be empty"));
        }
        if name.len() > 64 {
            return Err(Error::validation("role name must not exceed 64 characters"));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: OrganizationRoleId::new(),
            organization_id,
            name,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates the role name.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the name is empty or too long.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::validation("role name must not be empty"));
        }
        if name.len() > 64 {
            return Err(Error::validation("role name must not exceed 64 characters"));
        }
        self.name = name;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }

    /// Updates the role description.
    pub fn set_description(&mut self, description: Option<impl Into<String>>) {
        self.description = description.map(Into::into);
        self.updated_at = OffsetDateTime::now_utc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_role() {
        let role = OrganizationRole::new(OrganizationId::new(), "moderator", None::<String>)
            .expect("valid role");
        assert_eq!(role.name, "moderator");
    }

    #[test]
    fn empty_role_name_rejected() {
        assert!(OrganizationRole::new(OrganizationId::new(), "", None::<String>).is_err());
    }
}
