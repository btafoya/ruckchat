//! SQLx implementation of [`MessageReadRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::MessageReadRepository;
use ruckchat_id::{MessageId, UserId};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// SQLx-backed message read-state repository.
#[derive(Debug, Clone)]
pub struct MessageReadRepositorySqlx {
    pool: PgPool,
}

impl MessageReadRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageReadRepository for MessageReadRepositorySqlx {
    async fn mark_read(&self, user_id: UserId, message_ids: &[MessageId]) -> Result<()> {
        if message_ids.is_empty() {
            return Ok(());
        }
        let message_ids: Vec<Uuid> = message_ids.iter().map(|id| id.as_uuid()).collect();
        sqlx::query!(
            "INSERT INTO message_reads (user_id, message_id)
             SELECT $1, msg_id FROM UNNEST($2::uuid[]) AS msg_id
             ON CONFLICT DO NOTHING",
            user_id.as_uuid(),
            &message_ids,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn unread_counts_by_conversation(
        &self,
        user_id: UserId,
        conversation_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, i64>> {
        if conversation_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = sqlx::query!(
            r#"SELECT m.conversation_id AS "conversation_id!", COUNT(*) AS "count!"
             FROM messages m
             WHERE m.conversation_id = ANY($1)
               AND m.deleted_at IS NULL
               AND m.author_id != $2
               AND NOT EXISTS (
                 SELECT 1 FROM message_reads r
                 WHERE r.user_id = $2 AND r.message_id = m.id
               )
             GROUP BY m.conversation_id"#,
            conversation_ids,
            user_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows
            .into_iter()
            .map(|row| (row.conversation_id, row.count))
            .collect())
    }

    async fn unread_message_ids(
        &self,
        user_id: UserId,
        message_ids: &[MessageId],
    ) -> Result<std::collections::HashSet<MessageId>> {
        if message_ids.is_empty() {
            return Ok(std::collections::HashSet::new());
        }
        let ids: Vec<Uuid> = message_ids.iter().map(|id| id.as_uuid()).collect();
        let rows = sqlx::query!(
            r#"SELECT id AS "id!" FROM messages
             WHERE id = ANY($1)
               AND NOT EXISTS (
                 SELECT 1 FROM message_reads r WHERE r.user_id = $2 AND r.message_id = messages.id
               )"#,
            &ids,
            user_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows
            .into_iter()
            .map(|row| MessageId::from_uuid(row.id))
            .collect())
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    ruckchat_common::Error::Internal(err.to_string())
}
