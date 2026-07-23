//! SQLx implementation of [`TeamMembershipRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{TeamMembership, TeamMembershipRepository, TeamRole};
use ruckchat_id::{TeamId, UserId};
use sqlx::PgPool;
use std::str::FromStr;

/// SQLx-backed team membership repository.
#[derive(Debug, Clone)]
pub struct TeamMembershipRepositorySqlx {
    pool: PgPool,
}

impl TeamMembershipRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamMembershipRepository for TeamMembershipRepositorySqlx {
    async fn create(&self, membership: &TeamMembership) -> Result<()> {
        sqlx::query!(
            "INSERT INTO team_memberships (team_id, user_id, role, joined_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (team_id, user_id) DO NOTHING",
            membership.team_id.as_uuid(),
            membership.user_id.as_uuid(),
            membership.role.to_string(),
            membership.joined_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn list_by_team(&self, team_id: TeamId) -> Result<Vec<TeamMembership>> {
        let rows = sqlx::query_as!(
            MembershipRow,
            "SELECT team_id, user_id, role, joined_at FROM team_memberships WHERE team_id = $1 ORDER BY joined_at",
            team_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_membership).collect())
    }
}

#[derive(sqlx::FromRow)]
struct MembershipRow {
    team_id: uuid::Uuid,
    user_id: uuid::Uuid,
    role: String,
    joined_at: time::OffsetDateTime,
}

fn into_membership(row: MembershipRow) -> TeamMembership {
    TeamMembership {
        team_id: TeamId::from_uuid(row.team_id),
        user_id: UserId::from_uuid(row.user_id),
        role: TeamRole::from_str(&row.role).unwrap_or_default(),
        joined_at: row.joined_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("team membership".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
