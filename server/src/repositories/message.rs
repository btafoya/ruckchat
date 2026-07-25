//! SQLx implementation of [`MessageRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{File, Message, MessageCursor, MessageRepository};
use ruckchat_id::{FileId, MessageId, OrganizationId, UserId};
use sqlx::PgPool;
use std::collections::HashMap;
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

    /// Loads attached files for a batch of messages in a single query, keyed
    /// by message id.
    async fn load_attachments(&self, message_ids: &[Uuid]) -> Result<HashMap<Uuid, Vec<File>>> {
        if message_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = sqlx::query!(
            "SELECT mf.message_id, f.id, f.organization_id, f.uploaded_by, f.file_name, f.mime_type, f.size_bytes, f.storage_path, f.thumbnail_path, f.created_at
             FROM message_files mf
             JOIN files f ON f.id = mf.file_id
             WHERE mf.message_id = ANY($1)
             ORDER BY f.created_at",
            message_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        let mut attachments: HashMap<Uuid, Vec<File>> = HashMap::new();
        for row in rows {
            let file = File {
                id: FileId::from_uuid(row.id),
                organization_id: OrganizationId::from_uuid(row.organization_id),
                uploaded_by: UserId::from_uuid(row.uploaded_by),
                file_name: row.file_name,
                mime_type: row.mime_type,
                size_bytes: row.size_bytes,
                storage_path: row.storage_path,
                thumbnail_path: row.thumbnail_path,
                created_at: row.created_at,
            };
            attachments.entry(row.message_id).or_default().push(file);
        }
        Ok(attachments)
    }

    /// Converts rows into messages and batch-attaches their files.
    async fn hydrate(&self, rows: Vec<MessageRow>) -> Result<Vec<Message>> {
        let mut messages = rows
            .into_iter()
            .map(into_message)
            .collect::<Result<Vec<_>>>()?;
        let ids: Vec<Uuid> = messages.iter().map(|m| m.id.as_uuid()).collect();
        let mut attachments = self.load_attachments(&ids).await?;
        for message in &mut messages {
            message.attachments = attachments
                .remove(&message.id.as_uuid())
                .unwrap_or_default();
        }
        Ok(messages)
    }
}

#[async_trait]
impl MessageRepository for MessageRepositorySqlx {
    async fn create(&self, message: &Message) -> Result<()> {
        let mentioned_user_ids: Vec<uuid::Uuid> = message
            .mentioned_user_ids
            .iter()
            .map(|id| id.as_uuid())
            .collect();
        sqlx::query!(
            "INSERT INTO messages (id, conversation_id, conversation_type, parent_id, author_id, content, mentioned_user_ids, created_at, updated_at, deleted_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT DO NOTHING",
            message.id.as_uuid(),
            message.conversation_id,
            message.conversation_type.to_string(),
            message.parent_id.map(|id| id.as_uuid()),
            message.author_id.as_uuid(),
            message.content,
            &mentioned_user_ids,
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
            "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.author_id
             WHERE m.id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let mut message = into_message(row)?;
        let mut attachments = self.load_attachments(&[message.id.as_uuid()]).await?;
        message.attachments = attachments
            .remove(&message.id.as_uuid())
            .unwrap_or_default();
        Ok(Some(message))
    }

    async fn list_before(
        &self,
        conversation_id: Uuid,
        before: Option<MessageCursor>,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let rows = match before {
            Some(cursor) => {
                sqlx::query_as!(
                    MessageRow,
                    "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
                     FROM messages m
                     LEFT JOIN users u ON u.id = m.author_id
                     WHERE m.conversation_id = $1 AND m.deleted_at IS NULL
                       AND (m.created_at, m.id) < ($2, $3)
                     ORDER BY m.created_at DESC, m.id DESC
                     LIMIT $4",
                    conversation_id,
                    cursor.created_at,
                    cursor.id.as_uuid(),
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as!(
                    MessageRow,
                    "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
                     FROM messages m
                     LEFT JOIN users u ON u.id = m.author_id
                     WHERE m.conversation_id = $1 AND m.deleted_at IS NULL
                     ORDER BY m.created_at DESC, m.id DESC
                     LIMIT $2",
                    conversation_id,
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(map_sqlx_err)?;

        let mut messages = self.hydrate(rows).await?;
        messages.reverse();
        Ok(messages)
    }

    async fn list_after(
        &self,
        conversation_id: Uuid,
        after: MessageCursor,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let rows = sqlx::query_as!(
            MessageRow,
            "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.author_id
             WHERE m.conversation_id = $1 AND m.deleted_at IS NULL
               AND (m.created_at, m.id) > ($2, $3)
             ORDER BY m.created_at ASC, m.id ASC
             LIMIT $4",
            conversation_id,
            after.created_at,
            after.id.as_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        self.hydrate(rows).await
    }

    async fn list_replies_before(
        &self,
        parent_id: MessageId,
        before: Option<MessageCursor>,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let rows = match before {
            Some(cursor) => {
                sqlx::query_as!(
                    MessageRow,
                    "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
                     FROM messages m
                     LEFT JOIN users u ON u.id = m.author_id
                     WHERE m.parent_id = $1 AND m.deleted_at IS NULL
                       AND (m.created_at, m.id) < ($2, $3)
                     ORDER BY m.created_at DESC, m.id DESC
                     LIMIT $4",
                    parent_id.as_uuid(),
                    cursor.created_at,
                    cursor.id.as_uuid(),
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as!(
                    MessageRow,
                    "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
                     FROM messages m
                     LEFT JOIN users u ON u.id = m.author_id
                     WHERE m.parent_id = $1 AND m.deleted_at IS NULL
                     ORDER BY m.created_at DESC, m.id DESC
                     LIMIT $2",
                    parent_id.as_uuid(),
                    limit
                )
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(map_sqlx_err)?;

        let mut messages = self.hydrate(rows).await?;
        messages.reverse();
        Ok(messages)
    }

    async fn list_replies_after(
        &self,
        parent_id: MessageId,
        after: MessageCursor,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let rows = sqlx::query_as!(
            MessageRow,
            "SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.author_id
             WHERE m.parent_id = $1 AND m.deleted_at IS NULL
               AND (m.created_at, m.id) > ($2, $3)
             ORDER BY m.created_at ASC, m.id ASC
             LIMIT $4",
            parent_id.as_uuid(),
            after.created_at,
            after.id.as_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        self.hydrate(rows).await
    }

    async fn update(&self, message: &Message) -> Result<()> {
        let mentioned_user_ids: Vec<uuid::Uuid> = message
            .mentioned_user_ids
            .iter()
            .map(|id| id.as_uuid())
            .collect();
        sqlx::query!(
            "UPDATE messages
             SET conversation_id = $2, conversation_type = $3, parent_id = $4, author_id = $5,
                 content = $6, mentioned_user_ids = $7, created_at = $8, updated_at = $9, deleted_at = $10
             WHERE id = $1",
            message.id.as_uuid(),
            message.conversation_id,
            message.conversation_type.to_string(),
            message.parent_id.map(|id| id.as_uuid()),
            message.author_id.as_uuid(),
            message.content,
            &mentioned_user_ids,
            message.created_at,
            message.updated_at,
            message.deleted_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn search(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let rows = sqlx::query_as!(
            MessageRow,
            r#"SELECT m.id, m.conversation_id, m.conversation_type, m.parent_id, m.author_id, u.display_name AS author_display_name, m.content, m.mentioned_user_ids, m.created_at, m.updated_at, m.deleted_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.author_id
             WHERE m.deleted_at IS NULL
               AND m.content_tsv @@ plainto_tsquery('english', $1)
               AND (
                 (
                   m.conversation_type = 'channel'
                   AND m.conversation_id IN (
                     SELECT id FROM channels
                     WHERE organization_id = $2 AND is_private = false
                   )
                 )
                 OR
                 (
                   m.conversation_type = 'channel'
                   AND m.conversation_id IN (
                     SELECT c.id FROM channels c
                     JOIN channel_memberships cm ON cm.channel_id = c.id
                     WHERE c.organization_id = $2 AND c.is_private = true AND cm.user_id = $3
                   )
                 )
                 OR
                 (
                   m.conversation_type = 'dm'
                   AND m.conversation_id IN (
                     SELECT dmc.id FROM direct_message_conversations dmc
                     JOIN dm_members dmm ON dmm.conversation_id = dmc.id
                     WHERE dmc.organization_id = $2 AND dmm.user_id = $3
                   )
                 )
               )
             ORDER BY m.created_at DESC
             LIMIT $4 OFFSET $5"#,
            query,
            organization_id.as_uuid(),
            caller_id.as_uuid(),
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        self.hydrate(rows).await
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: uuid::Uuid,
    conversation_id: Uuid,
    conversation_type: String,
    parent_id: Option<uuid::Uuid>,
    author_id: uuid::Uuid,
    author_display_name: Option<String>,
    content: String,
    mentioned_user_ids: Vec<uuid::Uuid>,
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
        author_display_name: row.author_display_name,
        content: row.content,
        mentioned_user_ids: row
            .mentioned_user_ids
            .into_iter()
            .map(ruckchat_id::UserId::from_uuid)
            .collect(),
        attachments: Vec::new(),
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
