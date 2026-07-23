//! RuckChat API request and response models for the migration tool.

use ruckchat_id::{FileId, UserId};
use serde::{Deserialize, Serialize};

/// Response from a successful login.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    /// Session token to use as a Bearer token.
    pub token: String,
    /// Authenticated user.
    pub user: LoginUser,
}

/// Authenticated user returned by login.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginUser {
    /// User identifier.
    pub id: UserId,
    /// Email address.
    pub email: String,
    /// Display name.
    pub display_name: String,
}

/// Import snapshot request.
#[derive(Debug, Clone, Serialize)]
pub struct ImportRequest {
    /// Migration snapshot.
    pub data: crate::ruckchat::client::MigrationData,
    /// Validate without writing when true.
    pub dry_run: bool,
}

/// Import snapshot response.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportResponse {
    /// Rows inserted or updated.
    pub inserted: usize,
    /// Rows skipped because they already existed.
    pub skipped: usize,
}

/// File upload response.
#[derive(Debug, Clone, Deserialize)]
pub struct FileResponse {
    /// File identifier.
    pub id: FileId,
    /// Original file name.
    pub file_name: String,
    /// MIME type.
    pub mime_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
}
