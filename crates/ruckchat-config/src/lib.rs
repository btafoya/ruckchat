//! Configuration primitives for RuckChat applications.
//!
//! This crate provides environment/file based configuration and validation.
//! Service-specific wiring (e.g., binding HTTP listeners) lives in the server
//! crate, not here.

use ruckchat_common::validate_email;
use ruckchat_id::UserId;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// Base application configuration shared across server, desktop, mobile, and plugins.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Human-readable application name.
    pub app_name: String,
    /// Deployment environment, e.g. `development`, `production`.
    pub environment: Environment,
    /// Base URL of the service.
    pub base_url: String,
    /// Log level directive, e.g. `info`.
    pub log_level: String,
    /// Whether the MCP endpoint is enabled.
    #[serde(default = "default_mcp_enabled")]
    pub mcp_enabled: bool,
    /// Whether MCP `post_message` requires explicit confirmation.
    #[serde(default = "default_mcp_require_confirmation")]
    pub mcp_require_confirmation: bool,
}

#[must_use]
fn default_mcp_enabled() -> bool {
    true
}

#[must_use]
fn default_mcp_require_confirmation() -> bool {
    true
}

impl AppConfig {
    /// Loads configuration from the default sources: `ruckchat.toml`, environment
    /// variables prefixed with `RUCKCHAT_`, and then `.env` overrides.
    ///
    /// # Errors
    ///
    /// Returns a config builder error if sources cannot be read or merged.
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("ruckchat").required(false))
            .add_source(
                config::Environment::with_prefix("RUCKCHAT")
                    .separator("__")
                    .try_parsing(true),
            )
            .set_default("app_name", "RuckChat")?
            .set_default("environment", "development")?
            .set_default("base_url", "http://localhost:3000")?
            .set_default("log_level", "info")?
            .set_default("mcp_enabled", true)?
            .set_default("mcp_require_confirmation", true)?
            .build()?;

        cfg.try_deserialize()
    }

    /// Validates the loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns a human-readable message describing the first invalid field.
    pub fn validate(&self) -> Result<(), String> {
        if self.app_name.is_empty() {
            return Err("app_name must not be empty".into());
        }
        if self.base_url.is_empty() {
            return Err("base_url must not be empty".into());
        }
        Ok(())
    }
}

/// Deployment environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    /// Local development.
    #[default]
    Development,
    /// Automated tests.
    Test,
    /// Staging.
    Staging,
    /// Production.
    Production,
}

/// Database connection configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Connection URL; password should be provided via a secret source.
    pub url: SecretString,
    /// Maximum connection pool size.
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Creates a config from a URL string.
    #[must_use]
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: SecretString::new(url.into().into()),
            max_connections: 10,
        }
    }

    /// Returns the URL with credentials exposed.
    ///
    /// # Security
    ///
    /// Only use this when handing the URL to trusted infrastructure such as a
    /// connection pool; never log it.
    #[must_use]
    pub fn url_exposed(&self) -> &str {
        self.url.expose_secret()
    }
}

/// Identity of an authenticated user provided by an upstream auth layer.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// Internal user identifier.
    pub id: UserId,
    /// Verified email address.
    pub email: String,
}

impl AuthenticatedUser {
    /// Validates the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns a validation message if the email is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if !validate_email(&self.email) {
            return Err(format!("invalid email: {}", self.email));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_config_hides_secret() {
        let cfg = DatabaseConfig::from_url("postgres://user:secret@localhost/db");
        assert_eq!(cfg.url_exposed(), "postgres://user:secret@localhost/db");
        // `Debug` must not leak the secret.
        let debug = format!("{:?}", cfg);
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn app_config_default_loads() {
        let cfg = AppConfig::load().expect("load default config");
        assert_eq!(cfg.environment, Environment::Development);
        assert!(!cfg.app_name.is_empty());
    }

    #[test]
    fn authenticated_user_validates_email() {
        let user = AuthenticatedUser {
            id: UserId::new(),
            email: "bad-email".into(),
        };
        assert!(user.validate().is_err());
    }
}
