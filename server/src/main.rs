//! RuckChat server binary.
//!
//! Loads configuration, connects to PostgreSQL, runs pending migrations, builds
//! the Axum application state, and starts the HTTP server. Also provides data
//! migration subcommands.

use clap::{Parser, Subcommand};
use ruckchat_config::{AppConfig, ConfigError, DatabaseConfig, default_config_path};
use ruckchat_server::{connect_database, handlers::router, migrate, state::AppState};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "ruckchat-server", version, about = "RuckChat server")]
struct Args {
    /// Path to the YAML configuration file.
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Write a default configuration file and exit. Optionally accepts a path;
    /// otherwise uses the platform default.
    #[arg(long, value_name = "PATH", num_args = 0..=1)]
    init_config: Option<Option<PathBuf>>,

    /// Subcommand to run.
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the HTTP server (default).
    Run,
    /// Export or import RuckChat data.
    Migrate {
        #[command(subcommand)]
        subcommand: MigrateSubcommand,
    },
}

#[derive(Subcommand, Debug)]
enum MigrateSubcommand {
    /// Export all data to a JSON file.
    Export {
        /// Output file path.
        #[arg(short, long, value_name = "PATH")]
        output: PathBuf,
    },
    /// Import data from a JSON file.
    Import {
        /// Input file path.
        #[arg(short, long, value_name = "PATH")]
        input: PathBuf,
        /// Validate the file without writing to the database.
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(init_config) = args.init_config {
        let path = init_config.unwrap_or_else(|| {
            default_config_path().unwrap_or_else(|err| {
                eprintln!("could not determine default config path: {err}");
                std::process::exit(1);
            })
        });
        match AppConfig::write_default_to(&path) {
            Ok(written) => {
                println!("wrote default configuration to {}", written.display());
                std::process::exit(0);
            }
            Err(err) => {
                eprintln!("failed to write default configuration: {err}");
                std::process::exit(1);
            }
        }
    }

    match args.command.unwrap_or(Command::Run) {
        Command::Run => {
            if let Err(err) = run(args.config).await {
                eprintln!("server failed: {err}");
                std::process::exit(1);
            }
        }
        Command::Migrate { subcommand } => {
            if let Err(err) = run_migrate(args.config, subcommand).await {
                eprintln!("migration failed: {err}");
                std::process::exit(1);
            }
        }
    }
}

async fn run(config_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;

    init_tracing(&config.log_level);

    let db_config = DatabaseConfig::from_url(config.database.url_exposed());
    let pool = connect_database(&db_config).await?;

    let state = AppState::from_config(pool, &config);

    let addr = parse_server_addr(&config.base_url).await?;
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on http://{addr}");

    let app = router(&config.web, &config.base_url).with_state(state);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn run_migrate(
    config_path: Option<PathBuf>,
    subcommand: MigrateSubcommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;
    init_tracing(&config.log_level);

    let db_config = DatabaseConfig::from_url(config.database.url_exposed());
    let pool = connect_database(&db_config).await?;

    match subcommand {
        MigrateSubcommand::Export { output } => {
            let data = migrate::export_to_file(&pool, &output).await?;
            println!(
                "exported {} users, {} organizations, {} channels, {} messages, {} roles, {} permissions, {} emoji, {} teams to {}",
                data.users.len(),
                data.organizations.len(),
                data.channels.len(),
                data.messages.len(),
                data.organization_roles.len(),
                data.permissions.len(),
                data.custom_emoji.len(),
                data.teams.len(),
                output.display()
            );
        }
        MigrateSubcommand::Import { input, dry_run } => {
            let data = migrate::read_migration_file(&input).await?;
            let counts = migrate::import(&pool, &data, dry_run).await?;
            if dry_run {
                println!(
                    "dry run: would skip {} existing rows and insert {} new rows",
                    counts.skipped, counts.inserted
                );
            } else {
                println!(
                    "imported {} rows, skipped {} existing rows",
                    counts.inserted, counts.skipped
                );
            }
        }
    }

    Ok(())
}

fn load_config(config_path: Option<PathBuf>) -> Result<AppConfig, ConfigError> {
    match config_path {
        Some(path) => AppConfig::load_from_path(path),
        None => AppConfig::load().map_err(|err| {
            if matches!(err, ConfigError::Read { .. }) {
                let default = default_config_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "the platform default path".into());
                ConfigError::Validation(format!(
                    "config file not found at {default}; create one with `ruckchat-server --init-config` or pass `--config <path>`"
                ))
            } else {
                err
            }
        }),
    }
}

fn init_tracing(log_level: &str) {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(log_level);
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn parse_server_addr(base_url: &str) -> Result<SocketAddr, String> {
    let url = url::Url::parse(base_url).map_err(|e| format!("invalid base_url: {e}"))?;
    let port = url.port().unwrap_or(3000);

    tokio::net::lookup_host(("0.0.0.0", port))
        .await
        .map_err(|e| format!("invalid server address: {e}"))?
        .next()
        .ok_or_else(|| "invalid server address: no addresses found".to_string())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_config_path() {
        let args = Args::parse_from(["ruckchat-server", "--config", "/etc/ruckchat/ruckchat.yaml"]);
        assert_eq!(
            args.config,
            Some(PathBuf::from("/etc/ruckchat/ruckchat.yaml"))
        );
        assert!(args.init_config.is_none());
        assert!(matches!(args.command, Some(Command::Run) | None));
    }

    #[test]
    fn parse_args_init_config() {
        let args = Args::parse_from(["ruckchat-server", "--init-config"]);
        assert_eq!(args.init_config, Some(None));
        assert!(args.config.is_none());
    }

    #[test]
    fn parse_args_init_config_with_path() {
        let args = Args::parse_from(["ruckchat-server", "--init-config", "./dev.yaml"]);
        assert_eq!(args.init_config, Some(Some(PathBuf::from("./dev.yaml"))));
    }

    #[test]
    fn parse_args_migrate_export() {
        let args = Args::parse_from([
            "ruckchat-server",
            "--config",
            "./ruckchat.yaml",
            "migrate",
            "export",
            "--output",
            "export.json",
        ]);
        assert_eq!(args.config, Some(PathBuf::from("./ruckchat.yaml")));
        match args.command {
            Some(Command::Migrate {
                subcommand: MigrateSubcommand::Export { output },
            }) => assert_eq!(output, PathBuf::from("export.json")),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_args_migrate_import() {
        let args = Args::parse_from([
            "ruckchat-server",
            "migrate",
            "import",
            "--input",
            "import.json",
            "--dry-run",
        ]);
        match args.command {
            Some(Command::Migrate {
                subcommand: MigrateSubcommand::Import { input, dry_run },
            }) => {
                assert_eq!(input, PathBuf::from("import.json"));
                assert!(dry_run);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
