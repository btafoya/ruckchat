//! RuckChat server binary.
//!
//! Loads configuration, connects to PostgreSQL, runs pending migrations, builds
//! the Axum application state, and starts the HTTP server.

use ruckchat_config::{AppConfig, ConfigError, DatabaseConfig, default_config_path};
use ruckchat_server::{connect_database, handlers::router, state::AppState};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let args = parse_args();

    if args.init_config {
        let path = args.config.unwrap_or_else(|| {
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

    if let Err(err) = run(args.config).await {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}

async fn run(config_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config = match config_path {
        Some(path) => AppConfig::load_from_path(path)?,
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
        })?,
    };

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

#[derive(Debug, Default)]
struct Args {
    /// Path to the YAML configuration file.
    config: Option<PathBuf>,
    /// Write a default configuration file and exit.
    init_config: bool,
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                let value = iter.next().unwrap_or_else(|| {
                    eprintln!("--config requires a path argument");
                    std::process::exit(1);
                });
                args.config = Some(PathBuf::from(value));
            }
            "--init-config" => {
                args.init_config = true;
            }
            "--help" | "-h" => {
                println!("Usage: ruckchat-server [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --config <PATH>    Path to ruckchat.yaml");
                println!("  --init-config      Write a default config file and exit");
                println!("  -h, --help         Print this help message");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(1);
            }
        }
    }
    args
}

fn init_tracing(log_level: &str) {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(log_level);
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn parse_server_addr(base_url: &str) -> Result<SocketAddr, String> {
    let url = url::Url::parse(base_url).map_err(|e| format!("invalid base_url: {e}"))?;
    let host = url.host_str().unwrap_or("127.0.0.1").to_string();
    let port = url.port().unwrap_or(3000);

    tokio::net::lookup_host((host, port))
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
        let args = parse_args_from(&["--config", "/etc/ruckchat/ruckchat.yaml"]);
        assert_eq!(
            args.config,
            Some(PathBuf::from("/etc/ruckchat/ruckchat.yaml"))
        );
        assert!(!args.init_config);
    }

    #[test]
    fn parse_args_init_config() {
        let args = parse_args_from(&["--init-config"]);
        assert!(args.init_config);
        assert!(args.config.is_none());
    }

    #[test]
    fn parse_args_config_with_init() {
        let args = parse_args_from(&["--init-config", "--config", "./dev.yaml"]);
        assert!(args.init_config);
        assert_eq!(args.config, Some(PathBuf::from("./dev.yaml")));
    }

    fn parse_args_from(input: &[&str]) -> Args {
        let mut args = Args::default();
        let mut iter = input.iter().map(|s| s.to_string());
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--config" => {
                    args.config = Some(PathBuf::from(iter.next().expect("value")));
                }
                "--init-config" => {
                    args.init_config = true;
                }
                _ => {}
            }
        }
        args
    }
}
