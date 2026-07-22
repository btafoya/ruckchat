//! SQLx implementation of [`FileRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{File, FileRepository};
use ruckchat_id::{FileId, OrganizationId};
use sqlx::PgPool;

/// SQLx-backed file metadata repository.
#[derive(Debug, Clone)]
pub struct FileRepositorySqlx {
    pool: PgPool,
}

impl FileRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRepository for FileRepositorySqlx {
    async fn create(&self, file: &File) -> Result<()> {
        sqlx::query!(
            "INSERT INTO files (id, organization_id, uploaded_by, file_name, mime_type, size_bytes, storage_path, thumbnail_path, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT DO NOTHING",
            file.id.as_uuid(),
            file.organization_id.as_uuid(),
            file.uploaded_by.as_uuid(),
            file.file_name,
            file.mime_type,
            file.size_bytes,
            file.storage_path,
            file.thumbnail_path,
            file.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: FileId) -> Result<Option<File>> {
        let row = sqlx::query_as!(
            FileRow,
            "SELECT id, organization_id, uploaded_by, file_name, mime_type, size_bytes, storage_path, thumbnail_path, created_at FROM files WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_file))
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<File>> {
        let rows = sqlx::query_as!(
            FileRow,
            "SELECT id, organization_id, uploaded_by, file_name, mime_type, size_bytes, storage_path, thumbnail_path, created_at FROM files WHERE organization_id = $1 ORDER BY created_at DESC",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_file).collect())
    }

    async fn attach_to_message(
        &self,
        message_id: ruckchat_id::MessageId,
        file_id: FileId,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO message_files (message_id, file_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            message_id.as_uuid(),
            file_id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct FileRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    uploaded_by: uuid::Uuid,
    file_name: String,
    mime_type: String,
    size_bytes: i64,
    storage_path: String,
    thumbnail_path: Option<String>,
    created_at: time::OffsetDateTime,
}

fn into_file(row: FileRow) -> File {
    File {
        id: FileId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        uploaded_by: ruckchat_id::UserId::from_uuid(row.uploaded_by),
        file_name: row.file_name,
        mime_type: row.mime_type,
        size_bytes: row.size_bytes,
        storage_path: row.storage_path,
        thumbnail_path: row.thumbnail_path,
        created_at: row.created_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("file".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
