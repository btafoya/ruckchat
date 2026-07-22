//! SQLx implementation of [`ChannelRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Channel, ChannelRepository};
use ruckchat_id::{ChannelId, OrganizationId};
use sqlx::PgPool;

/// SQLx-backed channel repository.
#[derive(Debug, Clone)]
pub struct ChannelRepositorySqlx {
    pool: PgPool,
}

impl ChannelRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelRepository for ChannelRepositorySqlx {
    async fn create(&self, channel: &Channel) -> Result<()> {
        let result = sqlx::query!(
            "INSERT INTO channels (id, organization_id, name, topic, purpose, is_private, is_archived, created_by, created_at, archived_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (organization_id, name) DO NOTHING",
            channel.id.as_uuid(),
            channel.organization_id.as_uuid(),
            channel.name,
            channel.topic,
            channel.purpose,
            channel.is_private,
            channel.archived_at.is_some(),
            channel.created_by.as_uuid(),
            channel.created_at,
            channel.archived_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        if result.rows_affected() == 0 {
            return Err(ruckchat_common::Error::Conflict(
                "channel name already exists".into(),
            ));
        }
        Ok(())
    }

    async fn update(&self, channel: &Channel) -> Result<()> {
        let rows = sqlx::query!(
            "UPDATE channels
             SET name = $2, topic = $3, purpose = $4, is_private = $5, is_archived = $6, archived_at = $7
             WHERE id = $1",
            channel.id.as_uuid(),
            channel.name,
            channel.topic,
            channel.purpose,
            channel.is_private,
            channel.archived_at.is_some(),
            channel.archived_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        if rows.rows_affected() == 0 {
            return Err(ruckchat_common::Error::NotFound("channel".into()));
        }
        Ok(())
    }

    async fn by_id(&self, id: ChannelId) -> Result<Option<Channel>> {
        let row = sqlx::query_as!(
            ChannelRow,
            "SELECT id, organization_id, name, topic, purpose, is_private, created_by, created_at, archived_at FROM channels WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_channel))
    }

    async fn list_by_organization(&self, organization_id: OrganizationId) -> Result<Vec<Channel>> {
        let rows = sqlx::query_as!(
            ChannelRow,
            "SELECT id, organization_id, name, topic, purpose, is_private, created_by, created_at, archived_at FROM channels WHERE organization_id = $1 ORDER BY name",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_channel).collect())
    }
}

#[derive(sqlx::FromRow)]
struct ChannelRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    name: String,
    topic: Option<String>,
    purpose: Option<String>,
    is_private: bool,
    created_by: uuid::Uuid,
    created_at: time::OffsetDateTime,
    archived_at: Option<time::OffsetDateTime>,
}

fn into_channel(row: ChannelRow) -> Channel {
    Channel {
        id: ChannelId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        name: row.name,
        topic: row.topic,
        purpose: row.purpose,
        is_private: row.is_private,
        created_by: ruckchat_id::UserId::from_uuid(row.created_by),
        created_at: row.created_at,
        archived_at: row.archived_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("channel".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
