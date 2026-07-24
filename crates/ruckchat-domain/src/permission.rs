//! Permission aggregate.

use ruckchat_common::{Error, Result};
use ruckchat_id::{OrganizationId, PermissionId};
use serde::{Deserialize, Serialize};

/// A permission that can be granted to a role within an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Internal permission identifier.
    pub id: PermissionId,
    /// Organization this permission belongs to.
    pub organization_id: OrganizationId,
    /// Machine-readable permission key.
    pub key: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

impl Permission {
    /// Creates a new permission after validating the key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the key is empty or contains invalid characters.
    pub fn new(
        organization_id: OrganizationId,
        key: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Result<Self> {
        let key = key.into();
        let description = description.map(Into::into);

        if key.is_empty() {
            return Err(Error::validation("permission key must not be empty"));
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::validation(
                "permission key must contain only letters, numbers, hyphens, and underscores",
            ));
        }

        Ok(Self {
            id: PermissionId::new(),
            organization_id,
            key,
            description,
        })
    }

    /// Updates the permission key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the key is empty or contains invalid characters.
    pub fn set_key(&mut self, key: impl Into<String>) -> Result<()> {
        let key = key.into();
        if key.is_empty() {
            return Err(Error::validation("permission key must not be empty"));
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::validation(
                "permission key must contain only letters, numbers, hyphens, and underscores",
            ));
        }
        self.key = key;
        Ok(())
    }

    /// Updates the permission description.
    pub fn set_description(&mut self, description: Option<impl Into<String>>) {
        self.description = description.map(Into::into);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_permission() {
        let perm = Permission::new(OrganizationId::new(), "manage_channels", None::<String>)
            .expect("valid permission");
        assert_eq!(perm.key, "manage_channels");
    }

    #[test]
    fn empty_permission_key_rejected() {
        assert!(Permission::new(OrganizationId::new(), "", None::<String>).is_err());
    }

    #[test]
    fn invalid_permission_key_rejected() {
        assert!(Permission::new(OrganizationId::new(), "manage channels", None::<String>).is_err());
    }
}
