//! RuckChat server binary.
//!
//! Loads configuration, connects to PostgreSQL, runs pending migrations, builds
//! the Axum application state, and starts the HTTP server.

use ruckchat_config::{AppConfig, DatabaseConfig};
use ruckchat_server::{connect_database, handlers::router, state::AppState};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    config.validate()?;

    init_tracing(&config.log_level);

    let db_config = DatabaseConfig::from_url(
        std::env::var("DATABASE_URL")
            .as_deref()
            .unwrap_or("postgres://ruckchat:ruckchat@localhost/ruckchat"),
    );
    let pool = connect_database(&db_config).await?;

    let secure_cookies = matches!(config.environment, ruckchat_config::Environment::Production);
    let state = AppState::from_pool(pool, secure_cookies);

    let addr = parse_server_addr(&config.base_url)?;
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on http://{addr}");

    let app = router().with_state(state);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing(log_level: &str) {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(log_level);
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn parse_server_addr(base_url: &str) -> Result<SocketAddr, String> {
    let url = url::Url::parse(base_url).map_err(|e| format!("invalid base_url: {e}"))?;
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(3000);
    format!("{host}:{port}")
        .parse()
        .map_err(|e| format!("invalid server address: {e}"))
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
