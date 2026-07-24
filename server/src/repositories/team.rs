//! SQLx implementation of [`TeamRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Team, TeamRepository};
use ruckchat_id::{OrganizationId, TeamId};
use sqlx::PgPool;

/// SQLx-backed team repository.
#[derive(Debug, Clone)]
pub struct TeamRepositorySqlx {
    pool: PgPool,
}

impl TeamRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamRepository for TeamRepositorySqlx {
    async fn create(&self, team: &Team) -> Result<()> {
        sqlx::query!(
            "INSERT INTO teams (id, organization_id, name, description, created_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (organization_id, name) DO NOTHING",
            team.id.as_uuid(),
            team.organization_id.as_uuid(),
            team.name,
            team.description,
            team.created_by.as_uuid(),
            team.created_at,
            team.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: TeamId) -> Result<Option<Team>> {
        let row = sqlx::query_as!(
            TeamRow,
            "SELECT id, organization_id, name, description, created_by, created_at, updated_at FROM teams WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_team))
    }

    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<Team>> {
        let rows = sqlx::query_as!(
            TeamRow,
            "SELECT id, organization_id, name, description, created_by, created_at, updated_at FROM teams WHERE organization_id = $1 ORDER BY name",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_team).collect())
    }

    async fn update(&self, team: &Team) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE teams SET name = $2, description = $3, updated_at = $4 WHERE id = $1",
            team.id.as_uuid(),
            team.name,
            team.description,
            team.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(ruckchat_common::Error::NotFound("team".into()));
        }
        Ok(())
    }

    async fn delete(&self, id: TeamId) -> Result<Option<()>> {
        let result = sqlx::query!("DELETE FROM teams WHERE id = $1", id.as_uuid())
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
struct TeamRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    name: String,
    description: Option<String>,
    created_by: uuid::Uuid,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

fn into_team(row: TeamRow) -> Team {
    Team {
        id: TeamId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        name: row.name,
        description: row.description,
        created_by: ruckchat_id::UserId::from_uuid(row.created_by),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("team".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
