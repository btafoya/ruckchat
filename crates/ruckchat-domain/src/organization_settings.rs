//! Organization settings aggregate.

use ruckchat_common::{
    Error, Result,
    time::OffsetDateTime,
    validation::{DEFAULT_MAX_FILE_SIZE_BYTES, DEFAULT_ORG_STORAGE_QUOTA_BYTES},
};
use ruckchat_id::OrganizationId;
use serde::{Deserialize, Serialize};

/// Per-organization limits and quotas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationSettings {
    /// Organization these settings apply to.
    pub organization_id: OrganizationId,
    /// Maximum size of a single uploaded file in bytes.
    pub max_file_size_bytes: i64,
    /// Total storage quota for the organization in bytes.
    pub storage_quota_bytes: i64,
    /// Timestamp of the last settings update.
    pub updated_at: OffsetDateTime,
}

impl OrganizationSettings {
    /// Creates settings for an organization with sensible defaults.
    pub fn new(organization_id: OrganizationId) -> Self {
        Self {
            organization_id,
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
            storage_quota_bytes: DEFAULT_ORG_STORAGE_QUOTA_BYTES,
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    /// Updates file and storage quotas.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] if quotas are not positive.
    pub fn set_quotas(&mut self, max_file_size_bytes: i64, storage_quota_bytes: i64) -> Result<()> {
        if max_file_size_bytes <= 0 {
            return Err(Error::validation("max file size must be positive"));
        }
        if storage_quota_bytes <= 0 {
            return Err(Error::validation("storage quota must be positive"));
        }
        self.max_file_size_bytes = max_file_size_bytes;
        self.storage_quota_bytes = storage_quota_bytes;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let settings = OrganizationSettings::new(OrganizationId::new());
        assert_eq!(settings.max_file_size_bytes, DEFAULT_MAX_FILE_SIZE_BYTES);
        assert_eq!(
            settings.storage_quota_bytes,
            DEFAULT_ORG_STORAGE_QUOTA_BYTES
        );
    }

    #[test]
    fn update_quotas() {
        let mut settings = OrganizationSettings::new(OrganizationId::new());
        settings.set_quotas(10_000, 100_000).expect("update quotas");
        assert_eq!(settings.max_file_size_bytes, 10_000);
        assert_eq!(settings.storage_quota_bytes, 100_000);
    }

    #[test]
    fn invalid_quotas_rejected() {
        let mut settings = OrganizationSettings::new(OrganizationId::new());
        assert!(settings.set_quotas(0, 100_000).is_err());
        assert!(settings.set_quotas(10_000, 0).is_err());
    }
}
