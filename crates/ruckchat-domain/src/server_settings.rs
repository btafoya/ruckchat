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
        }
    }
}
