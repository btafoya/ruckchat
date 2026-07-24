//! SQLx implementation of [`OrganizationRoleRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{OrganizationRole, OrganizationRoleRepository};
use ruckchat_id::{OrganizationId, OrganizationRoleId};
use sqlx::PgPool;

/// SQLx-backed custom organization role repository.
#[derive(Debug, Clone)]
pub struct OrganizationRoleRepositorySqlx {
    pool: PgPool,
}

impl OrganizationRoleRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRoleRepository for OrganizationRoleRepositorySqlx {
    async fn create(&self, role: &OrganizationRole) -> Result<()> {
        sqlx::query!(
            "INSERT INTO organization_roles (id, organization_id, name, description, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (organization_id, name) DO NOTHING",
            role.id.as_uuid(),
            role.organization_id.as_uuid(),
            role.name,
            role.description,
            role.created_at,
            role.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: OrganizationRoleId) -> Result<Option<OrganizationRole>> {
        let row = sqlx::query_as!(
            RoleRow,
            "SELECT id, organization_id, name, description, created_at, updated_at FROM organization_roles WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_role))
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationRole>> {
        let rows = sqlx::query_as!(
            RoleRow,
            "SELECT id, organization_id, name, description, created_at, updated_at FROM organization_roles WHERE organization_id = $1 ORDER BY name",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_role).collect())
    }

    async fn update(&self, role: &OrganizationRole) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE organization_roles SET name = $2, description = $3, updated_at = $4 WHERE id = $1",
            role.id.as_uuid(),
            role.name,
            role.description,
            role.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(ruckchat_common::Error::NotFound("organization role".into()));
        }
        Ok(())
    }

    async fn delete(&self, id: OrganizationRoleId) -> Result<Option<()>> {
        let result = sqlx::query!("DELETE FROM organization_roles WHERE id = $1", id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(if result.rows_affected() == 0 {
            None
        } else {
            Some(())
        })
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    name: String,
    description: Option<String>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

fn into_role(row: RoleRow) -> OrganizationRole {
    OrganizationRole {
        id: OrganizationRoleId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        name: row.name,
        description: row.description,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("organization role".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
