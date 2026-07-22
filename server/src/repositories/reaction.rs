//! SQLx implementation of [`ReactionRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Reaction, ReactionRepository};
use ruckchat_id::{MessageId, UserId};
use sqlx::PgPool;

/// SQLx-backed reaction repository.
#[derive(Debug, Clone)]
pub struct ReactionRepositorySqlx {
    pool: PgPool,
}

impl ReactionRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReactionRepository for ReactionRepositorySqlx {
    async fn create(&self, reaction: &Reaction) -> Result<()> {
        sqlx::query!(
            "INSERT INTO reactions (message_id, user_id, emoji, created_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (message_id, user_id, emoji) DO NOTHING",
            reaction.message_id.as_uuid(),
            reaction.user_id.as_uuid(),
            reaction.emoji,
            reaction.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn list_by_message(&self, message_id: MessageId) -> Result<Vec<Reaction>> {
        let rows = sqlx::query_as!(
            ReactionRow,
            "SELECT message_id, user_id, emoji, created_at FROM reactions WHERE message_id = $1 ORDER BY created_at",
            message_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_reaction).collect())
    }

    async fn delete(&self, message_id: MessageId, user_id: UserId, emoji: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
            message_id.as_uuid(),
            user_id.as_uuid(),
            emoji
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ReactionRow {
    message_id: uuid::Uuid,
    user_id: uuid::Uuid,
    emoji: String,
    created_at: time::OffsetDateTime,
}

fn into_reaction(row: ReactionRow) -> Reaction {
    Reaction {
        message_id: MessageId::from_uuid(row.message_id),
        user_id: UserId::from_uuid(row.user_id),
        emoji: row.emoji,
        created_at: row.created_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("reaction".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
