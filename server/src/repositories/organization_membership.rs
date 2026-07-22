//! SQLx implementation of [`OrganizationMembershipRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{OrganizationMembership, OrganizationMembershipRepository, Role};
use ruckchat_id::{OrganizationId, UserId};
use sqlx::PgPool;
use std::str::FromStr;

/// SQLx-backed organization membership repository.
#[derive(Debug, Clone)]
pub struct OrganizationMembershipRepositorySqlx {
    pool: PgPool,
}

impl OrganizationMembershipRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationMembershipRepository for OrganizationMembershipRepositorySqlx {
    async fn create(&self, membership: &OrganizationMembership) -> Result<()> {
        sqlx::query!(
            "INSERT INTO organization_memberships (user_id, organization_id, role, joined_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id, organization_id) DO UPDATE SET role = EXCLUDED.role",
            membership.user_id.as_uuid(),
            membership.organization_id.as_uuid(),
            membership.role.to_string(),
            membership.joined_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_ids(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationMembership>> {
        let row = sqlx::query_as!(
            MembershipRow,
            "SELECT user_id, organization_id, role, joined_at FROM organization_memberships WHERE user_id = $1 AND organization_id = $2",
            user_id.as_uuid(),
            organization_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_membership))
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationMembership>> {
        let rows = sqlx::query_as!(
            MembershipRow,
            "SELECT user_id, organization_id, role, joined_at FROM organization_memberships WHERE organization_id = $1 ORDER BY joined_at",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_membership).collect())
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<OrganizationMembership>> {
        let rows = sqlx::query_as!(
            MembershipRow,
            "SELECT user_id, organization_id, role, joined_at FROM organization_memberships WHERE user_id = $1 ORDER BY joined_at",
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_membership).collect())
    }

    async fn update_role(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        role: Role,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE organization_memberships SET role = $3 WHERE user_id = $1 AND organization_id = $2",
            user_id.as_uuid(),
            organization_id.as_uuid(),
            role.to_string(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn delete(&self, user_id: UserId, organization_id: OrganizationId) -> Result<()> {
        sqlx::query!(
            "DELETE FROM organization_memberships WHERE user_id = $1 AND organization_id = $2",
            user_id.as_uuid(),
            organization_id.as_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MembershipRow {
    user_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    role: String,
    joined_at: time::OffsetDateTime,
}

fn into_membership(row: MembershipRow) -> OrganizationMembership {
    OrganizationMembership {
        user_id: UserId::from_uuid(row.user_id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        role: Role::from_str(&row.role).unwrap_or_default(),
        joined_at: row.joined_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => {
            ruckchat_common::Error::NotFound("organization membership".into())
        }
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
