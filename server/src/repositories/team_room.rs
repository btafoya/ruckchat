//! SQLx implementation of [`TeamRoomRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{TeamRoom, TeamRoomRepository};
use ruckchat_id::{ChannelId, TeamId};
use sqlx::PgPool;

/// SQLx-backed team-room link repository.
#[derive(Debug, Clone)]
pub struct TeamRoomRepositorySqlx {
    pool: PgPool,
}

impl TeamRoomRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamRoomRepository for TeamRoomRepositorySqlx {
    async fn create(&self, link: &TeamRoom) -> Result<()> {
        sqlx::query!(
            "INSERT INTO team_rooms (team_id, channel_id, added_at)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
            link.team_id.as_uuid(),
            link.channel_id.as_uuid(),
            link.added_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn list_by_team(&self, team_id: TeamId) -> Result<Vec<TeamRoom>> {
        let rows = sqlx::query_as!(
            TeamRoomRow,
            "SELECT team_id, channel_id, added_at FROM team_rooms WHERE team_id = $1 ORDER BY added_at",
            team_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_team_room).collect())
    }
}

#[derive(sqlx::FromRow)]
struct TeamRoomRow {
    team_id: uuid::Uuid,
    channel_id: uuid::Uuid,
    added_at: time::OffsetDateTime,
}

fn into_team_room(row: TeamRoomRow) -> TeamRoom {
    TeamRoom {
        team_id: TeamId::from_uuid(row.team_id),
        channel_id: ChannelId::from_uuid(row.channel_id),
        added_at: row.added_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("team room".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
