//! CLI and YAML configuration for the migration tool.

use std::path::{Path, PathBuf};

use clap::Parser;
use serde::Deserialize;
use tracing::warn;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::interactive;

/// Command-line arguments.
#[derive(Debug, Clone, Parser)]
#[command(name = "rocketchat2ruckchat")]
#[command(about = "Migrate a RocketChat MongoDB dump into a RuckChat organization")]
pub struct Cli {
    /// Path to a YAML configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Actually write changes to the target RuckChat database.
    #[arg(long)]
    pub apply: bool,

    /// Email each migrated user their temporary password. Only meaningful
    /// alongside `--apply`; sending is a separate, explicit action from
    /// writing data since it cannot be undone.
    #[arg(long)]
    pub send_emails: bool,

    /// Always prompt for missing values even when a config file is supplied.
    #[arg(long)]
    pub interactive: bool,

    /// Run without writing anything and print a dry-run report.
    #[arg(long)]
    pub dry_run: bool,

    /// Path to the SQLite mapping store.
    #[arg(long)]
    pub mapping_store: Option<PathBuf>,
}

/// Source MongoDB configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SourceConfig {
    /// Connection string for the restored RocketChat MongoDB dump.
    #[serde(default)]
    pub mongo_uri: String,
    /// Database name within the Mongo connection.
    #[serde(default = "default_mongo_database")]
    pub database: String,
}

fn default_mongo_database() -> String {
    "rocketchat".into()
}

/// Target RuckChat configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TargetConfig {
    /// PostgreSQL connection string for the target RuckChat database.
    pub database_url: String,
    /// Target organization identifier. May already exist (its row is left
    /// untouched, `ON CONFLICT DO NOTHING`) or be created fresh.
    pub organization_id: Uuid,
    /// Organization display name, used only if `organization_id` doesn't
    /// already exist.
    pub organization_name: String,
    /// Organization slug, used only if `organization_id` doesn't already
    /// exist.
    pub organization_slug: String,
    /// Email identifying the migration's fallback/owner identity. Matched
    /// against existing target users by email first; a fresh account is
    /// created only when no match exists.
    pub admin_email: String,
    /// Directory RuckChat stores uploaded file bytes in. Must match the
    /// target server's `ruckchat.yaml` `files.directory`.
    pub upload_directory: String,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            organization_id: Uuid::nil(),
            organization_name: "Migrated Organization".into(),
            organization_slug: "migrated".into(),
            admin_email: String::new(),
            upload_directory: String::new(),
        }
    }
}

/// Postmark configuration for migration credential emails.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EmailConfig {
    /// Postmark server API token.
    #[serde(default)]
    pub server_token: String,
    /// From address used for credential emails.
    #[serde(default)]
    pub from_address: String,
}

/// Migration options.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OptionsConfig {
    /// Entity categories to migrate.
    pub scope: Vec<String>,
    /// Map existing RuckChat users by email when possible.
    pub map_existing_users: bool,
    /// Mark deactivated/inactive RocketChat users as deactivated.
    pub deactivate_deleted_users: bool,
    /// Archive rooms that were archived in RocketChat.
    pub archive_deleted_rooms: bool,
    /// Default to dry-run unless overridden.
    pub dry_run: bool,
}

impl OptionsConfig {
    /// Returns true if a scope category is enabled.
    #[must_use]
    pub fn has_scope(&self, category: &str) -> bool {
        self.scope.iter().any(|s| s == category)
    }
}

impl Default for OptionsConfig {
    fn default() -> Self {
        Self {
            scope: vec![
                "users".into(),
                "channels".into(),
                "messages".into(),
                "reactions".into(),
                "files".into(),
                "emoji".into(),
            ],
            map_existing_users: true,
            deactivate_deleted_users: true,
            archive_deleted_rooms: true,
            dry_run: true,
        }
    }
}

/// Raw on-disk configuration.
#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    source: Option<SourceConfig>,
    target: Option<TargetConfig>,
    email: Option<EmailConfig>,
    #[serde(default)]
    options: OptionsConfig,
    mapping_store: Option<PathBuf>,
}

/// Fully resolved, ready-to-run configuration.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// Source MongoDB configuration.
    pub source: SourceConfig,
    /// Target RuckChat configuration.
    pub target: TargetConfig,
    /// Postmark configuration, if credential emails are wanted.
    pub email: Option<EmailConfig>,
    /// Migration options.
    pub options: OptionsConfig,
    /// Path to the SQLite mapping store.
    pub mapping_store: PathBuf,
    /// True when `--apply` was supplied and this run may write data.
    pub apply: bool,
    /// True when migrated users should be emailed their credentials.
    pub send_emails: bool,
}

impl ResolvedConfig {
    /// Returns true when the current run is a dry run.
    #[must_use]
    pub fn is_dry_run(&self) -> bool {
        !self.apply || self.options.dry_run
    }

    /// Returns true if a scope category is enabled.
    #[must_use]
    pub fn has_scope(&self, category: &str) -> bool {
        self.options.scope.iter().any(|s| s == category)
    }
}

/// Loads configuration from the CLI, config file, and optional prompts.
pub fn resolve(cli: &Cli) -> Result<ResolvedConfig> {
    let mut file = load_file_config(cli.config.as_deref())?;

    let interactive = cli.interactive || cli.config.is_none();

    if interactive {
        interactive::prompt_source(&mut file.source)?;
        interactive::prompt_target(&mut file.target)?;
        if cli.send_emails {
            interactive::prompt_email(&mut file.email)?;
        }
        if file.mapping_store.is_none() {
            let default = default_mapping_store();
            let path = interactive::prompt_mapping_store(default)?;
            file.mapping_store = Some(path);
        }
    }

    let source = file
        .source
        .clone()
        .ok_or_else(|| Error::config("source configuration is required"))?;
    let target = file
        .target
        .clone()
        .ok_or_else(|| Error::config("target configuration is required"))?;

    if source.mongo_uri.is_empty() {
        return Err(Error::config("source.mongo_uri is required"));
    }
    if target.database_url.is_empty() {
        return Err(Error::config("target.database_url is required"));
    }
    if target.admin_email.is_empty() {
        return Err(Error::config("target.admin_email is required"));
    }
    if target.upload_directory.is_empty() {
        return Err(Error::config("target.upload_directory is required"));
    }

    let mapping_store = cli
        .mapping_store
        .clone()
        .or(file.mapping_store)
        .unwrap_or_else(default_mapping_store);

    let apply = cli.apply;
    if apply && interactive {
        interactive::confirm_apply()?;
    }

    if cli.dry_run && apply {
        warn!("--dry-run overrides --apply; no writes will occur");
    }

    if cli.send_emails && !apply {
        return Err(Error::config("--send-emails requires --apply"));
    }

    let mut options = file.options;
    if cli.dry_run {
        options.dry_run = true;
    }

    Ok(ResolvedConfig {
        source,
        target,
        email: file.email,
        options,
        mapping_store,
        apply,
        send_emails: cli.send_emails,
    })
}

fn load_file_config(path: Option<&Path>) -> Result<FileConfig> {
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };
    let content = std::fs::read_to_string(path)?;
    let config: FileConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn default_mapping_store() -> PathBuf {
    PathBuf::from("rocketchat2ruckchat.mapping.sqlite")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_includes_all_scopes() {
        let options = OptionsConfig::default();
        assert!(options.has_scope("users"));
        assert!(options.has_scope("channels"));
        assert!(options.has_scope("messages"));
    }

    fn sample_config(apply: bool, dry_run: bool) -> ResolvedConfig {
        ResolvedConfig {
            source: SourceConfig {
                mongo_uri: "mongodb://localhost:27017".into(),
                database: "rocketchat".into(),
            },
            target: TargetConfig {
                database_url: "postgres://localhost/ruckchat".into(),
                organization_id: Uuid::nil(),
                organization_name: "Migrated Organization".into(),
                organization_slug: "migrated".into(),
                admin_email: "admin@example.com".into(),
                upload_directory: "/tmp/uploads".into(),
            },
            email: None,
            options: OptionsConfig {
                dry_run,
                ..OptionsConfig::default()
            },
            mapping_store: default_mapping_store(),
            apply,
            send_emails: false,
        }
    }

    #[test]
    fn resolved_config_dry_run_without_apply() {
        let config = sample_config(false, true);
        assert!(config.is_dry_run());
    }

    #[test]
    fn resolved_config_apply_overrides_default_dry_run() {
        let config = sample_config(true, false);
        assert!(!config.is_dry_run());
    }
}
