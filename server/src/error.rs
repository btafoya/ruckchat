//! Server-specific error type.
//!
//! Services return [`ruckchat_common::Error`] directly because it already
//! covers the failure modes needed by HTTP handlers. This module exists to
//! provide a thin wrapper for errors that originate inside the server crate
//! itself, such as password hashing failures or invalid session tokens.

use ruckchat_common::Error as DomainError;

/// Errors produced by the server crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the domain/service layer.
    #[error(transparent)]
    Domain(#[from] DomainError),

    /// Password hashing or verification failed.
    #[error("password operation failed")]
    PasswordHash,

    /// Token generation failed.
    #[error("token generation failed")]
    TokenGeneration,
}

impl From<argon2::password_hash::Error> for Error {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::PasswordHash
    }
}

impl From<rand_core::Error> for Error {
    fn from(_: rand_core::Error) -> Self {
        Self::TokenGeneration
    }
}

/// Result alias for server operations.
pub type Result<T> = std::result::Result<T, Error>;
