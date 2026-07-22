//! SQLx implementation of [`OrganizationRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Organization, OrganizationRepository};
use ruckchat_id::{OrganizationId, UserId};
use sqlx::PgPool;

/// SQLx-backed organization repository.
#[derive(Debug, Clone)]
pub struct OrganizationRepositorySqlx {
    pool: PgPool,
}

impl OrganizationRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for OrganizationRepositorySqlx {
    async fn create(&self, organization: &Organization) -> Result<()> {
        sqlx::query!(
            "INSERT INTO organizations (id, name, slug, owner_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (slug) DO NOTHING",
            organization.id.as_uuid(),
            organization.name,
            organization.slug,
            organization.owner_id.as_uuid(),
            organization.created_at,
            organization.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: OrganizationId) -> Result<Option<Organization>> {
        let row = sqlx::query_as!(
            OrganizationRow,
            "SELECT id, name, slug, owner_id, created_at, updated_at FROM organizations WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_organization))
    }

    async fn by_slug(&self, slug: &str) -> Result<Option<Organization>> {
        let row = sqlx::query_as!(
            OrganizationRow,
            "SELECT id, name, slug, owner_id, created_at, updated_at FROM organizations WHERE slug = $1",
            slug
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_organization))
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Organization>> {
        let rows = sqlx::query_as!(
            OrganizationRow,
            "SELECT o.id, o.name, o.slug, o.owner_id, o.created_at, o.updated_at
             FROM organizations o
             JOIN organization_memberships m ON m.organization_id = o.id
             WHERE m.user_id = $1
             ORDER BY o.name",
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_organization).collect())
    }
}

#[derive(sqlx::FromRow)]
struct OrganizationRow {
    id: uuid::Uuid,
    name: String,
    slug: String,
    owner_id: uuid::Uuid,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

fn into_organization(row: OrganizationRow) -> Organization {
    Organization {
        id: OrganizationId::from_uuid(row.id),
        name: row.name,
        slug: row.slug,
        owner_id: UserId::from_uuid(row.owner_id),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("organization".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
