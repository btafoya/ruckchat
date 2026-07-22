//! Shared application state.
//!
//! This state is passed to HTTP handlers in later phases. It holds the
//! database pool and a typed collection of repository implementations.

use sqlx::PgPool;

/// Application state shared across HTTP handlers and background tasks.
#[derive(Debug, Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub pool: PgPool,
}

impl AppState {
    /// Creates state from a connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
