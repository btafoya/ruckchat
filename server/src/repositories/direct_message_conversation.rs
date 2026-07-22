//! SQLx implementation of [`DirectMessageConversationRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{DirectMessageConversation, DirectMessageConversationRepository};
use ruckchat_id::{DirectMessageConversationId, OrganizationId, UserId};
use sqlx::PgPool;

/// SQLx-backed direct message conversation repository.
#[derive(Debug, Clone)]
pub struct DirectMessageConversationRepositorySqlx {
    pool: PgPool,
}

impl DirectMessageConversationRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DirectMessageConversationRepository for DirectMessageConversationRepositorySqlx {
    async fn create(&self, conversation: &DirectMessageConversation) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        sqlx::query!(
            "INSERT INTO direct_message_conversations (id, organization_id, created_at)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
            conversation.id.as_uuid(),
            conversation.organization_id.as_uuid(),
            conversation.created_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;

        for member_id in &conversation.member_ids {
            sqlx::query!(
                "INSERT INTO dm_members (conversation_id, user_id)
                 VALUES ($1, $2)
                 ON CONFLICT DO NOTHING",
                conversation.id.as_uuid(),
                member_id.as_uuid(),
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;
        }

        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(
        &self,
        id: DirectMessageConversationId,
    ) -> Result<Option<DirectMessageConversation>> {
        let conversation_row = sqlx::query_as!(
            ConversationRow,
            "SELECT id, organization_id, created_at FROM direct_message_conversations WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        let Some(conversation_row) = conversation_row else {
            return Ok(None);
        };

        let members = sqlx::query_scalar!(
            "SELECT user_id FROM dm_members WHERE conversation_id = $1 ORDER BY user_id",
            id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(Some(into_conversation(conversation_row, members)))
    }

    async fn list_by_user_and_organization(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<DirectMessageConversation>> {
        let rows = sqlx::query_as!(
            ConversationRow,
            "SELECT c.id, c.organization_id, c.created_at
             FROM direct_message_conversations c
             JOIN dm_members m ON m.conversation_id = c.id
             WHERE c.organization_id = $1 AND m.user_id = $2
             ORDER BY c.created_at DESC",
            organization_id.as_uuid(),
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        let mut conversations = Vec::with_capacity(rows.len());
        for row in rows {
            let members = sqlx::query_scalar!(
                "SELECT user_id FROM dm_members WHERE conversation_id = $1 ORDER BY user_id",
                row.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

            conversations.push(into_conversation(row, members));
        }

        Ok(conversations)
    }
}

#[derive(sqlx::FromRow)]
struct ConversationRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    created_at: time::OffsetDateTime,
}

fn into_conversation(
    row: ConversationRow,
    member_uuids: Vec<uuid::Uuid>,
) -> DirectMessageConversation {
    DirectMessageConversation {
        id: DirectMessageConversationId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        member_ids: member_uuids.into_iter().map(UserId::from_uuid).collect(),
        created_at: row.created_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("dm conversation".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
