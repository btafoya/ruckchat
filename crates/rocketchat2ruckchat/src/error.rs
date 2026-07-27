//! Error type for the migration tool.

/// A unified error returned by the migration tool.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration file could not be read or parsed.
    #[error("config error: {0}")]
    Config(String),

    /// I/O failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parsing failed.
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// SQLite mapping store failure.
    #[error("mapping store error: {0}")]
    MappingStore(#[from] rusqlite::Error),

    /// MongoDB source failure.
    #[error("mongodb error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    /// PostgreSQL target failure.
    #[error("postgres error: {0}")]
    Postgres(#[from] sqlx::Error),

    /// The shared migrate crate rejected the snapshot or import.
    #[error("migrate error: {0}")]
    Migrate(#[from] ruckchat_migrate::MigrateError),

    /// Sending a credential email failed.
    #[error("email error: {0}")]
    Email(#[from] ruckchat_email::EmailError),

    /// Invalid input from the operator.
    #[error("input error: {0}")]
    Input(String),

    /// Interactive prompt failed.
    #[error("prompt error: {0}")]
    Prompt(#[from] dialoguer::Error),

    /// A migration stage produced inconsistent data.
    #[error("transform error: {0}")]
    Transform(String),

    /// An internal invariant was violated.
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Creates a transform error.
    #[must_use]
    pub fn transform(message: impl Into<String>) -> Self {
        Self::Transform(message.into())
    }

    /// Creates an internal error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

/// Result alias for the migration tool.
pub type Result<T> = std::result::Result<T, Error>;
