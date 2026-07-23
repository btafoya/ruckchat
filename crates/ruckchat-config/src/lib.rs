//! Configuration primitives for RuckChat applications.
//!
//! This crate provides a single source of truth for server runtime configuration:
//! a YAML file read once at startup. Service-specific wiring (e.g. binding HTTP
//! listeners) lives in the server crate, not here.

use ruckchat_common::validate_email;
use ruckchat_id::UserId;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::info;

/// Default configuration file name.
pub const CONFIG_FILE_NAME: &str = "ruckchat.yaml";

/// Base application configuration shared across server, desktop, mobile, and plugins.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    /// Human-readable application name.
    pub app_name: String,
    /// Deployment environment, e.g. `development`, `production`.
    pub environment: Environment,
    /// Base URL of the service.
    pub base_url: String,
    /// Log level directive, e.g. `info`.
    pub log_level: String,
    /// Database connection configuration.
    pub database: DatabaseConfig,
    /// MCP endpoint configuration.
    #[serde(default)]
    pub mcp: McpConfig,
    /// Native plugin configuration.
    #[serde(default)]
    pub plugins: PluginConfig,
}

/// MCP endpoint configuration.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct McpConfig {
    /// Whether the MCP endpoint is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Whether MCP `post_message` requires explicit confirmation.
    #[serde(default = "default_true")]
    pub require_confirmation: bool,
}

/// Native plugin configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PluginConfig {
    /// Directory containing native plugin dynamic libraries.
    #[serde(default = "default_plugin_dir")]
    pub directory: String,
}

#[must_use]
fn default_true() -> bool {
    true
}

#[must_use]
fn default_plugin_dir() -> String {
    "/var/lib/ruckchat/plugins".into()
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_confirmation: true,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            directory: default_plugin_dir(),
        }
    }
}

/// Error loading the configuration file.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The default configuration directory or path could not be determined.
    #[error("could not determine default config path for this platform")]
    DefaultPathUnknown,
    /// Failed to read the configuration file.
    #[error("failed to read config file at {path}: {source}")]
    Read {
        /// Configuration file path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// Failed to parse the configuration file.
    #[error("failed to parse config file at {path}: {source}")]
    Parse {
        /// Configuration file path.
        path: PathBuf,
        /// Underlying YAML parse error.
        #[source]
        source: yaml_serde::Error,
    },
    /// Configuration validation failed.
    #[error("invalid configuration: {0}")]
    Validation(String),
}

impl AppConfig {
    /// Loads configuration from the platform default path.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when the default path cannot be determined, the
    /// file cannot be read, or the file is invalid.
    pub fn load() -> Result<Self, ConfigError> {
        let path = default_config_path()?;
        Self::load_from_path(&path)
    }

    /// Loads configuration from the supplied path.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when the file cannot be read, parsed, or validated.
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        let config: Self = yaml_serde::from_str(&content).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        // Ensure the stored database URL never leaks through Debug; SecretString already
        // handles this, but we still need the config to be internally consistent.
        config.validate()?;

        info!(path = %path.display(), "loaded configuration");
        Ok(config)
    }

    /// Writes a fully-commented default configuration file to the given path.
    ///
    /// The file is intended for administrators to edit. It contains every
    /// currently supported key plus commented placeholders for future phases.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Read`] when the parent directory cannot be created
    /// or the file cannot be written.
    pub fn write_default_to(path: impl AsRef<Path>) -> Result<PathBuf, ConfigError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ConfigError::Read {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::write(path, DEFAULT_CONFIG).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        info!(path = %path.display(), "wrote default configuration");
        Ok(path.to_path_buf())
    }

    /// Validates the loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns a human-readable message describing the first invalid field.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.app_name.is_empty() {
            return Err(ConfigError::Validation("app_name must not be empty".into()));
        }
        if self.base_url.is_empty() {
            return Err(ConfigError::Validation("base_url must not be empty".into()));
        }
        if self.database.url.expose_secret().is_empty() {
            return Err(ConfigError::Validation(
                "database.url must not be empty".into(),
            ));
        }
        Ok(())
    }
}

/// Returns the platform-specific default configuration path.
///
/// # Errors
///
/// Returns [`ConfigError::DefaultPathUnknown`] when the platform is not recognized.
pub fn default_config_path() -> Result<PathBuf, ConfigError> {
    if cfg!(target_os = "linux") {
        Ok(PathBuf::from("/etc/ruckchat").join(CONFIG_FILE_NAME))
    } else if cfg!(target_os = "macos") {
        Ok(PathBuf::from("/Library/Application Support/RuckChat").join(CONFIG_FILE_NAME))
    } else if cfg!(target_os = "windows") {
        let program_data =
            std::env::var("ProgramData").map_err(|_| ConfigError::DefaultPathUnknown)?;
        Ok(PathBuf::from(program_data)
            .join("RuckChat")
            .join(CONFIG_FILE_NAME))
    } else {
        Err(ConfigError::DefaultPathUnknown)
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
#[serde(rename_all = "snake_case")]
pub struct DatabaseConfig {
    /// Connection URL; password should be provided via a secret source.
    pub url: SecretString,
    /// Maximum connection pool size.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[must_use]
fn default_max_connections() -> u32 {
    10
}

impl DatabaseConfig {
    /// Creates a config from a URL string.
    #[must_use]
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: SecretString::new(url.into().into()),
            max_connections: default_max_connections(),
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

/// Fully-commented default configuration file contents.
const DEFAULT_CONFIG: &str = r#"# RuckChat server runtime configuration.
#
# This file is the single source of truth for server settings. It is read once
# at startup and is not reloaded automatically. Edit this file and restart the
# service to apply changes.
#
# Generate a fresh copy with:
#   ruckchat-server --init-config
# Or write to an explicit path with:
#   ruckchat-server --init-config ./ruckchat.yaml

# Human-readable application name.
app_name: "RuckChat"

# Deployment environment: development | test | staging | production.
environment: "development"

# Public base URL used for links, cookies, and WebSocket discovery.
base_url: "http://localhost:3000"

# Server log level directive: trace | debug | info | warn | error.
log_level: "info"

# PostgreSQL connection settings.
database:
  # Connection URL. Keep this secret.
  url: "postgres://ruckchat:ruckchat@localhost/ruckchat"
  # Maximum size of the connection pool.
  max_connections: 10

# MCP server settings.
mcp:
  # Whether the MCP endpoint is exposed at /mcp/v1/sse.
  enabled: true
  # Whether MCP post_message tool calls require explicit confirmation.
  require_confirmation: true

# Native plugin dynamic-library directory.
plugins:
  directory: "/var/lib/ruckchat/plugins"

# Placeholders for future phases. These keys are ignored today.
# retention:
#   message_history_days: 365
#
# federation:
#   enabled: false
#   domain: ""
#
# limits:
#   max_organizations: 0
#   max_users_per_organization: 0
#   max_channels_per_organization: 0
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn database_config_hides_secret() {
        let cfg = DatabaseConfig::from_url("postgres://user:secret@localhost/db");
        assert_eq!(cfg.url_exposed(), "postgres://user:secret@localhost/db");
        // `Debug` must not leak the secret.
        let debug = format!("{:?}", cfg);
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn app_config_loads_from_valid_yaml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ruckchat.yaml");
        let mut file = std::fs::File::create(&path).expect("create file");
        file.write_all(
            r#"
app_name: "RuckChat Test"
environment: "test"
base_url: "http://127.0.0.1:3900"
log_level: "debug"
database:
  url: "postgres://test:test@localhost/test"
  max_connections: 5
mcp:
  enabled: false
  require_confirmation: false
plugins:
  directory: "/tmp/plugins"
"#
            .as_bytes(),
        )
        .expect("write yaml");

        let cfg = AppConfig::load_from_path(&path).expect("load config");
        assert_eq!(cfg.app_name, "RuckChat Test");
        assert_eq!(cfg.environment, Environment::Test);
        assert_eq!(cfg.base_url, "http://127.0.0.1:3900");
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(
            cfg.database.url_exposed(),
            "postgres://test:test@localhost/test"
        );
        assert_eq!(cfg.database.max_connections, 5);
        assert!(!cfg.mcp.enabled);
        assert!(!cfg.mcp.require_confirmation);
        assert_eq!(cfg.plugins.directory, "/tmp/plugins");
    }

    #[test]
    fn app_config_uses_defaults_for_optional_sections() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ruckchat.yaml");
        let mut file = std::fs::File::create(&path).expect("create file");
        file.write_all(
            r#"
app_name: "RuckChat"
environment: "development"
base_url: "http://localhost:3000"
log_level: "info"
database:
  url: "postgres://ruckchat:ruckchat@localhost/ruckchat"
"#
            .as_bytes(),
        )
        .expect("write yaml");

        let cfg = AppConfig::load_from_path(&path).expect("load config");
        assert!(cfg.mcp.enabled);
        assert!(cfg.mcp.require_confirmation);
        assert_eq!(cfg.plugins.directory, "/var/lib/ruckchat/plugins");
        assert_eq!(cfg.database.max_connections, 10);
    }

    #[test]
    fn missing_config_file_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.yaml");
        let err = AppConfig::load_from_path(&path).expect_err("should fail");
        assert!(format!("{err}").contains("failed to read config file"));
    }

    #[test]
    fn invalid_yaml_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bad.yaml");
        std::fs::write(&path, "app_name: [").expect("write bad yaml");
        let err = AppConfig::load_from_path(&path).expect_err("should fail");
        assert!(format!("{err}").contains("failed to parse config file"));
    }

    #[test]
    fn validation_rejects_empty_required_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("empty.yaml");
        let mut file = std::fs::File::create(&path).expect("create file");
        file.write_all(
            r#"
app_name: ""
environment: "development"
base_url: "http://localhost:3000"
log_level: "info"
database:
  url: "postgres://ruckchat:ruckchat@localhost/ruckchat"
"#
            .as_bytes(),
        )
        .expect("write yaml");

        let err = AppConfig::load_from_path(&path).expect_err("should fail");
        assert!(format!("{err}").contains("app_name must not be empty"));
    }

    #[test]
    fn write_default_config_creates_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ruckchat.yaml");
        let written = AppConfig::write_default_to(&path).expect("write default");
        assert_eq!(written, path);
        let content = std::fs::read_to_string(&path).expect("read file");
        assert!(content.contains("app_name:"));
        assert!(content.contains("database:"));
        assert!(content.contains("mcp:"));
        assert!(content.contains("plugins:"));
        // Should be loadable.
        AppConfig::load_from_path(&path).expect("load generated config");
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
