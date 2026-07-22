//! Shared error type used across RuckChat crates.

/// A generic result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The shared error type for the RuckChat workspace.
///
/// Domain services should use this for transport-friendly failures. For
/// unexpected/internal errors, prefer returning a concrete error type or using
/// a tracing span.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Input failed validation.
    #[error("validation failed: {message}")]
    Validation {
        /// Human-readable validation message.
        message: String,
    },

    /// A resource was not found.
    #[error("resource not found: {0}")]
    NotFound(String),

    /// The caller lacks permission for the operation.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// An authentication-related failure.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// A conflict such as a duplicate unique key.
    #[error("conflict: {0}")]
    Conflict(String),

    /// An external or internal service failure.
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Convenience constructor for validation errors.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = Error::validation("email is required");
        assert_eq!(err.to_string(), "validation failed: email is required");
    }
}
