//! SQLx implementation of [`ChannelMembershipRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{ChannelMembership, ChannelMembershipRepository};
use ruckchat_id::{ChannelId, UserId};
use sqlx::PgPool;

/// SQLx-backed channel membership repository.
#[derive(Debug, Clone)]
pub struct ChannelMembershipRepositorySqlx {
    pool: PgPool,
}

impl ChannelMembershipRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelMembershipRepository for ChannelMembershipRepositorySqlx {
    async fn create(&self, membership: &ChannelMembership) -> Result<()> {
        sqlx::query!(
            "INSERT INTO channel_memberships (user_id, channel_id, joined_at)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
            membership.user_id.as_uuid(),
            membership.channel_id.as_uuid(),
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
        channel_id: ChannelId,
    ) -> Result<Option<ChannelMembership>> {
        let row = sqlx::query_as!(
            MembershipRow,
            "SELECT user_id, channel_id, joined_at FROM channel_memberships WHERE user_id = $1 AND channel_id = $2",
            user_id.as_uuid(),
            channel_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_membership))
    }

    async fn list_by_channel(&self, channel_id: ChannelId) -> Result<Vec<ChannelMembership>> {
        let rows = sqlx::query_as!(
            MembershipRow,
            "SELECT user_id, channel_id, joined_at FROM channel_memberships WHERE channel_id = $1 ORDER BY joined_at",
            channel_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_membership).collect())
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<ChannelMembership>> {
        let rows = sqlx::query_as!(
            MembershipRow,
            "SELECT user_id, channel_id, joined_at FROM channel_memberships WHERE user_id = $1 ORDER BY joined_at",
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_membership).collect())
    }

    async fn delete(&self, user_id: UserId, channel_id: ChannelId) -> Result<()> {
        sqlx::query!(
            "DELETE FROM channel_memberships WHERE user_id = $1 AND channel_id = $2",
            user_id.as_uuid(),
            channel_id.as_uuid(),
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
    channel_id: uuid::Uuid,
    joined_at: time::OffsetDateTime,
}

fn into_membership(row: MembershipRow) -> ChannelMembership {
    ChannelMembership {
        user_id: UserId::from_uuid(row.user_id),
        channel_id: ChannelId::from_uuid(row.channel_id),
        joined_at: row.joined_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("channel membership".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
