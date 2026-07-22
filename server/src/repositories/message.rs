//! SQLx implementation of [`MessageRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Message, MessageRepository};
use ruckchat_id::MessageId;
use sqlx::PgPool;
use uuid::Uuid;

/// SQLx-backed message repository.
#[derive(Debug, Clone)]
pub struct MessageRepositorySqlx {
    pool: PgPool,
}

impl MessageRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for MessageRepositorySqlx {
    async fn create(&self, message: &Message) -> Result<()> {
        sqlx::query!(
            "INSERT INTO messages (id, conversation_id, conversation_type, parent_id, author_id, content, created_at, updated_at, deleted_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT DO NOTHING",
            message.id.as_uuid(),
            message.conversation_id,
            message.conversation_type.to_string(),
            message.parent_id.map(|id| id.as_uuid()),
            message.author_id.as_uuid(),
            message.content,
            message.created_at,
            message.updated_at,
            message.deleted_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: MessageId) -> Result<Option<Message>> {
        let row = sqlx::query_as!(
            MessageRow,
            "SELECT id, conversation_id, conversation_type, parent_id, author_id, content, created_at, updated_at, deleted_at FROM messages WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_message).transpose()?)
    }

    async fn list_by_conversation(
        &self,
        conversation_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let rows = sqlx::query_as!(
            MessageRow,
            "SELECT id, conversation_id, conversation_type, parent_id, author_id, content, created_at, updated_at, deleted_at
             FROM messages
             WHERE conversation_id = $1 AND deleted_at IS NULL
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
            conversation_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        rows.into_iter()
            .map(into_message)
            .collect::<Result<Vec<_>>>()
    }

    async fn update(&self, message: &Message) -> Result<()> {
        sqlx::query!(
            "UPDATE messages
             SET conversation_id = $2, conversation_type = $3, parent_id = $4, author_id = $5,
                 content = $6, created_at = $7, updated_at = $8, deleted_at = $9
             WHERE id = $1",
            message.id.as_uuid(),
            message.conversation_id,
            message.conversation_type.to_string(),
            message.parent_id.map(|id| id.as_uuid()),
            message.author_id.as_uuid(),
            message.content,
            message.created_at,
            message.updated_at,
            message.deleted_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: uuid::Uuid,
    conversation_id: Uuid,
    conversation_type: String,
    parent_id: Option<uuid::Uuid>,
    author_id: uuid::Uuid,
    content: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    deleted_at: Option<time::OffsetDateTime>,
}

fn into_message(row: MessageRow) -> Result<Message> {
    let conversation_type = row
        .conversation_type
        .parse::<ruckchat_domain::ConversationType>()
        .map_err(|_| ruckchat_common::Error::Internal("invalid conversation_type".into()))?;

    Ok(Message {
        id: MessageId::from_uuid(row.id),
        conversation_id: row.conversation_id,
        conversation_type,
        parent_id: row.parent_id.map(MessageId::from_uuid),
        author_id: ruckchat_id::UserId::from_uuid(row.author_id),
        content: row.content,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    })
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("message".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
