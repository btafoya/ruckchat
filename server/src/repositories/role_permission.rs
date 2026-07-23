//! SQLx implementation of [`RolePermissionRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{OrganizationRolePermission, RolePermissionRepository};
use ruckchat_id::{OrganizationRoleId, PermissionId};
use sqlx::PgPool;

/// SQLx-backed role-permission grant repository.
#[derive(Debug, Clone)]
pub struct RolePermissionRepositorySqlx {
    pool: PgPool,
}

impl RolePermissionRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RolePermissionRepository for RolePermissionRepositorySqlx {
    async fn create(&self, grant: &OrganizationRolePermission) -> Result<()> {
        sqlx::query!(
            "INSERT INTO organization_role_permissions (role_id, permission_id)
             VALUES ($1, $2)
             ON CONFLICT DO NOTHING",
            grant.role_id.as_uuid(),
            grant.permission_id.as_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn list_by_role(&self, role_id: OrganizationRoleId) -> Result<Vec<PermissionId>> {
        let rows = sqlx::query_scalar!(
            "SELECT permission_id FROM organization_role_permissions WHERE role_id = $1",
            role_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(PermissionId::from_uuid).collect())
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => {
            ruckchat_common::Error::NotFound("role permission grant".into())
        }
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
