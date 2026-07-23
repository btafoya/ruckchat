//! Error type for the migration tool.

use std::path::PathBuf;

use reqwest::StatusCode;

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

    /// RocketChat API failure.
    #[error("rocketchat api error: {message} (status {status:?})")]
    RocketChat {
        /// HTTP status code, if one was returned.
        status: Option<StatusCode>,
        /// Human-readable message.
        message: String,
    },

    /// RuckChat API failure.
    #[error("ruckchat api error: {message} (status {status:?})")]
    RuckChat {
        /// HTTP status code, if one was returned.
        status: Option<StatusCode>,
        /// Human-readable message.
        message: String,
    },

    /// HTTP request failed.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

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

    /// Creates a RocketChat API error from an optional status and message.
    #[must_use]
    pub fn rocketchat(status: Option<StatusCode>, message: impl Into<String>) -> Self {
        Self::RocketChat {
            status,
            message: message.into(),
        }
    }

    /// Creates a RuckChat API error from an optional status and message.
    #[must_use]
    pub fn ruckchat(status: Option<StatusCode>, message: impl Into<String>) -> Self {
        Self::RuckChat {
            status,
            message: message.into(),
        }
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

/// Errors that can occur while resolving a file path.
#[derive(Debug, thiserror::Error)]
#[error("path not found: {0}")]
pub struct PathError(pub PathBuf);
