//! Writes a migration snapshot directly into a target RuckChat PostgreSQL
//! database, bypassing the REST admin-import endpoint entirely.
//!
//! Uploaded file bytes are written into the same directory RuckChat's own
//! `FileService` uses, with the same naming convention
//! (`<upload_dir>/<file_id>`, flat, no extension), so the `files.storage_path`
//! rows this writes line up with what `FileService` expects to find on disk.

use std::collections::HashMap;
use std::path::PathBuf;

use ruckchat_id::FileId;
use ruckchat_migrate::{ImportCounts, MigrationData};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;

/// Connection to the target RuckChat PostgreSQL database.
pub struct PostgresTarget {
    pool: PgPool,
    upload_directory: PathBuf,
}

impl PostgresTarget {
    /// Connects to the target database.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::Postgres`] when the connection cannot
    /// be established.
    pub async fn connect(database_url: &str, upload_directory: impl Into<PathBuf>) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self {
            pool,
            upload_directory: upload_directory.into(),
        })
    }

    /// Writes a file's bytes to the configured upload directory and returns
    /// the resulting storage path.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::Io`] when the directory or file cannot
    /// be written.
    pub async fn write_file_bytes(&self, file_id: FileId, bytes: &[u8]) -> Result<String> {
        tokio::fs::create_dir_all(&self.upload_directory).await?;
        let path = self.upload_directory.join(file_id.as_uuid().to_string());
        tokio::fs::write(&path, bytes).await?;
        Ok(path.to_string_lossy().into_owned())
    }

    /// Returns every existing user's email (lowercased) mapped to their id,
    /// so the transform stage can attach migrated data to pre-existing
    /// accounts by email instead of always minting new ones.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::Postgres`] when the query fails.
    pub async fn existing_users_by_email(&self) -> Result<HashMap<String, Uuid>> {
        let rows = sqlx::query!("SELECT id, email FROM users")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| (row.email.to_ascii_lowercase(), row.id))
            .collect())
    }

    /// Imports a migration snapshot, delegating to the shared transactional,
    /// idempotent import logic RuckChat's own export/import CLI and admin
    /// endpoint use.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::Migrate`] when the snapshot is
    /// inconsistent, or [`crate::error::Error::Postgres`] when a write fails.
    pub async fn import(&self, data: &MigrationData, dry_run: bool) -> Result<ImportCounts> {
        Ok(ruckchat_migrate::import(&self.pool, data, dry_run).await?)
    }
}
