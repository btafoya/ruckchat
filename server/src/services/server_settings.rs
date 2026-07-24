//! Server-wide settings service.

use ruckchat_common::Result;
use ruckchat_domain::{ServerSettings, ServerSettingsRepository};
use ruckchat_id::UserId;
use std::sync::Arc;

/// Dependencies required by [`ServerSettingsService`].
#[derive(Clone)]
pub struct ServerSettingsServiceDeps {
    /// Server settings repository.
    pub repository: Arc<dyn ServerSettingsRepository + Send + Sync>,
    /// Runtime YAML overrides.
    pub overrides: ServerSettingsOverride,
}

/// YAML-provided overrides for soft server settings.
#[derive(Debug, Clone, Default)]
pub struct ServerSettingsOverride {
    /// Override for maintenance mode.
    pub maintenance_mode_enabled: Option<bool>,
    /// Override for default max file size.
    pub default_max_file_size_bytes: Option<i64>,
    /// Override for default storage quota.
    pub default_storage_quota_bytes: Option<i64>,
    /// Override for allowed signup domains.
    pub allowed_signup_domains: Option<Vec<String>>,
    /// Override for whether new user registrations are allowed.
    pub allow_registration: Option<bool>,
    /// Override for whether the server-side spell checker is enabled.
    pub spelling_enabled: Option<bool>,
    /// Override for the default spell-checker language tag.
    pub spelling_default_language: Option<String>,
}

/// Reads and updates server-wide settings.
#[derive(Clone)]
pub struct ServerSettingsService {
    deps: ServerSettingsServiceDeps,
}

impl ServerSettingsService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: ServerSettingsServiceDeps) -> Self {
        Self { deps }
    }

    /// Loads merged server settings.
    ///
    /// Precedence: YAML overrides > database values > hard-coded defaults.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Internal`] for repository failures.
    pub async fn load(&self) -> Result<ServerSettings> {
        let mut settings = self.deps.repository.load().await?;
        if let Some(value) = self.deps.overrides.maintenance_mode_enabled {
            settings.maintenance_mode_enabled = value;
        }
        if let Some(value) = self.deps.overrides.default_max_file_size_bytes {
            settings.default_max_file_size_bytes = value;
        }
        if let Some(value) = self.deps.overrides.default_storage_quota_bytes {
            settings.default_storage_quota_bytes = value;
        }
        if let Some(value) = self.deps.overrides.allowed_signup_domains.clone() {
            settings.allowed_signup_domains = value;
        }
        if let Some(value) = self.deps.overrides.allow_registration {
            settings.allow_registration = value;
        }
        if let Some(value) = self.deps.overrides.spelling_enabled {
            settings.spelling_enabled = value;
        }
        if let Some(value) = self.deps.overrides.spelling_default_language.clone() {
            settings.spelling_default_language = value;
        }
        Ok(settings)
    }

    /// Persists new server settings.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Internal`] for repository failures.
    pub async fn save(&self, settings: &ServerSettings, updated_by: UserId) -> Result<()> {
        self.deps.repository.save(settings, updated_by).await
    }
}
