//! SQLx implementation of [`PermissionRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Permission, PermissionRepository};
use ruckchat_id::{OrganizationId, PermissionId};
use sqlx::PgPool;

/// SQLx-backed permission repository.
#[derive(Debug, Clone)]
pub struct PermissionRepositorySqlx {
    pool: PgPool,
}

impl PermissionRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepository for PermissionRepositorySqlx {
    async fn create(&self, permission: &Permission) -> Result<()> {
        sqlx::query!(
            "INSERT INTO permissions (id, organization_id, key, description)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (organization_id, key) DO NOTHING",
            permission.id.as_uuid(),
            permission.organization_id.as_uuid(),
            permission.key,
            permission.description,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Permission>> {
        let rows = sqlx::query_as!(
            PermissionRow,
            "SELECT id, organization_id, key, description FROM permissions WHERE organization_id = $1 ORDER BY key",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_permission).collect())
    }
}

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    key: String,
    description: Option<String>,
}

fn into_permission(row: PermissionRow) -> Permission {
    Permission {
        id: PermissionId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        key: row.key,
        description: row.description,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("permission".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
