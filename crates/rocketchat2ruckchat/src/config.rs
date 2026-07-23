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
#[command(about = "Migrate a RocketChat workspace to a RuckChat organization")]
pub struct Cli {
    /// Path to a YAML configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Actually write changes to the target RuckChat server.
    #[arg(long)]
    pub apply: bool,

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

/// Source RocketChat configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SourceConfig {
    /// RocketChat base URL.
    pub url: String,
    /// Authentication method.
    pub auth: RocketAuthConfig,
}

/// RocketChat authentication variants.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketAuthConfig {
    /// Personal access token authentication.
    pub pat: Option<RocketPatAuth>,
    /// Username/password authentication.
    pub login: Option<RocketLoginAuth>,
}

/// RocketChat personal access token credentials.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketPatAuth {
    /// RocketChat user identifier.
    pub user_id: String,
    /// RocketChat personal access token.
    pub auth_token: String,
}

/// RocketChat username/password credentials.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketLoginAuth {
    /// RocketChat username.
    pub username: String,
    /// RocketChat password.
    pub password: String,
}

/// Target RuckChat configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TargetConfig {
    /// RuckChat base URL.
    pub url: String,
    /// Authentication method.
    pub auth: RuckAuthConfig,
    /// Target organization identifier.
    pub organization_id: Uuid,
}

/// RuckChat authentication variants.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuckAuthConfig {
    /// Email/password login.
    #[serde(default)]
    pub login: RuckLoginAuth,
}

/// RuckChat email/password credentials.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuckLoginAuth {
    /// User email.
    pub email: String,
    /// User password.
    pub password: String,
}

/// Migration options.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OptionsConfig {
    /// Entity categories to migrate.
    pub scope: Vec<String>,
    /// Map existing RuckChat users by email when possible.
    pub map_existing_users: bool,
    /// Mark deleted RocketChat users as deactivated.
    pub deactivate_deleted_users: bool,
    /// Archive rooms that were deleted in RocketChat.
    pub archive_deleted_rooms: bool,
    /// Skip messages that were deleted in RocketChat.
    pub skip_deleted_messages: bool,
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
                "rooms".into(),
                "messages".into(),
                "reactions".into(),
                "files".into(),
                "roles".into(),
                "permissions".into(),
                "emoji".into(),
                "teams".into(),
            ],
            map_existing_users: true,
            deactivate_deleted_users: true,
            archive_deleted_rooms: true,
            skip_deleted_messages: true,
            dry_run: true,
        }
    }
}

/// Raw on-disk configuration.
#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    source: Option<SourceConfig>,
    target: Option<TargetConfig>,
    #[serde(default)]
    options: OptionsConfig,
    mapping_store: Option<PathBuf>,
}

/// Fully resolved, ready-to-run configuration.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// RocketChat source configuration.
    pub source: SourceConfig,
    /// RuckChat target configuration.
    pub target: TargetConfig,
    /// Migration options.
    pub options: OptionsConfig,
    /// Path to the SQLite mapping store.
    pub mapping_store: PathBuf,
    /// True when `--apply` was supplied and this run may write data.
    pub apply: bool,
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

    let mut options = file.options;
    if cli.dry_run {
        options.dry_run = true;
    }

    Ok(ResolvedConfig {
        source,
        target,
        options,
        mapping_store,
        apply,
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
        assert!(options.has_scope("rooms"));
        assert!(options.has_scope("messages"));
    }

    #[test]
    fn resolved_config_dry_run_without_apply() {
        let config = ResolvedConfig {
            source: SourceConfig {
                url: "https://rc.example.com".into(),
                auth: RocketAuthConfig::default(),
            },
            target: TargetConfig {
                url: "http://localhost:3000".into(),
                auth: RuckAuthConfig {
                    login: RuckLoginAuth {
                        email: "admin@example.com".into(),
                        password: "secret".into(),
                    },
                },
                organization_id: Uuid::nil(),
            },
            options: OptionsConfig::default(),
            mapping_store: default_mapping_store(),
            apply: false,
        };
        assert!(config.is_dry_run());
    }

    #[test]
    fn resolved_config_apply_overrides_default_dry_run() {
        let config = ResolvedConfig {
            source: SourceConfig {
                url: "https://rc.example.com".into(),
                auth: RocketAuthConfig::default(),
            },
            target: TargetConfig {
                url: "http://localhost:3000".into(),
                auth: RuckAuthConfig {
                    login: RuckLoginAuth {
                        email: "admin@example.com".into(),
                        password: "secret".into(),
                    },
                },
                organization_id: Uuid::nil(),
            },
            options: OptionsConfig {
                dry_run: false,
                ..OptionsConfig::default()
            },
            mapping_store: default_mapping_store(),
            apply: true,
        };
        assert!(!config.is_dry_run());
    }
}
