//! Server-wide settings aggregate.

use serde::{Deserialize, Serialize};

/// Typed server-wide settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServerSettings {
    /// Whether the server is in maintenance mode.
    pub maintenance_mode_enabled: bool,
    /// Default maximum file upload size in bytes.
    pub default_max_file_size_bytes: i64,
    /// Default storage quota in bytes.
    pub default_storage_quota_bytes: i64,
    /// Allowed email domains for signup, empty means unrestricted.
    pub allowed_signup_domains: Vec<String>,
    /// Whether new user registrations are allowed.
    pub allow_registration: bool,
    /// Whether the server-side spell checker is enabled.
    pub spelling_enabled: bool,
    /// Default language tag for the spell checker.
    pub spelling_default_language: String,
}

impl ServerSettings {
    /// Creates settings with sensible defaults.
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            maintenance_mode_enabled: false,
            default_max_file_size_bytes: 25 * 1024 * 1024,
            default_storage_quota_bytes: 10 * 1024 * 1024 * 1024,
            allowed_signup_domains: Vec::new(),
            allow_registration: true,
            spelling_enabled: true,
            spelling_default_language: "en-US".to_string(),
        }
    }
}
