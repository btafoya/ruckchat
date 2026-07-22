//! RuckChat server crate.
//!
//! This crate contains the service layer, SQLx repository implementations, and
//! application wiring for the RuckChat server. HTTP handlers and the WebSocket
//! server are added in later phases.

pub mod error;
pub mod handlers;
pub mod repositories;
pub mod services;
pub mod state;

#[cfg(test)]
pub mod testing;

use ruckchat_config::DatabaseConfig;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing::{info, instrument};

/// Shared application errors used by services and handlers.
pub use error::Error;

/// Result alias for server operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Builds a PostgreSQL connection pool from the supplied configuration and runs
/// pending migrations.
///
/// # Errors
///
/// Returns an error when the database URL is invalid, the server is
/// unreachable, or migrations fail.
#[instrument]
pub async fn connect_database(config: &DatabaseConfig) -> sqlx::Result<sqlx::Pool<sqlx::Postgres>> {
    let url = config.url_exposed();
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url)
        .await?;

    info!("running pending database migrations");
    ruckchat_migrations::migrator().run(&pool).await?;

    Ok(pool)
}
