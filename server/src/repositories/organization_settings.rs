//! SQLx implementation of [`OrganizationSettingsRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{OrganizationSettings, OrganizationSettingsRepository};
use ruckchat_id::OrganizationId;
use sqlx::PgPool;

/// SQLx-backed organization settings repository.
#[derive(Debug, Clone)]
pub struct OrganizationSettingsRepositorySqlx {
    pool: PgPool,
}

impl OrganizationSettingsRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationSettingsRepository for OrganizationSettingsRepositorySqlx {
    async fn create(&self, settings: &OrganizationSettings) -> Result<()> {
        sqlx::query!(
            "INSERT INTO organization_settings (organization_id, max_file_size_bytes, storage_quota_bytes, updated_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (organization_id) DO UPDATE SET
                 max_file_size_bytes = EXCLUDED.max_file_size_bytes,
                 storage_quota_bytes = EXCLUDED.storage_quota_bytes,
                 updated_at = EXCLUDED.updated_at",
            settings.organization_id.as_uuid(),
            settings.max_file_size_bytes,
            settings.storage_quota_bytes,
            settings.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_organization_id(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationSettings>> {
        let row = sqlx::query_as!(
            SettingsRow,
            "SELECT organization_id, max_file_size_bytes, storage_quota_bytes, updated_at FROM organization_settings WHERE organization_id = $1",
            organization_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_settings))
    }

    async fn update(&self, settings: &OrganizationSettings) -> Result<()> {
        sqlx::query!(
            "UPDATE organization_settings SET max_file_size_bytes = $2, storage_quota_bytes = $3, updated_at = $4 WHERE organization_id = $1",
            settings.organization_id.as_uuid(),
            settings.max_file_size_bytes,
            settings.storage_quota_bytes,
            settings.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SettingsRow {
    organization_id: uuid::Uuid,
    max_file_size_bytes: i64,
    storage_quota_bytes: i64,
    updated_at: time::OffsetDateTime,
}

fn into_settings(row: SettingsRow) -> OrganizationSettings {
    let mut settings = OrganizationSettings::new(OrganizationId::from_uuid(row.organization_id));
    // Defaults are overwritten by the database values via set_quotas, which cannot fail here.
    let _ = settings.set_quotas(row.max_file_size_bytes, row.storage_quota_bytes);
    settings.updated_at = row.updated_at;
    settings
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("organization settings".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
