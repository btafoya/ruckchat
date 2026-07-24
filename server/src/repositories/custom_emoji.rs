//! SQLx implementation of [`CustomEmojiRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{CustomEmoji, CustomEmojiRepository};
use ruckchat_id::{CustomEmojiId, OrganizationId};
use sqlx::PgPool;

/// SQLx-backed custom emoji repository.
#[derive(Debug, Clone)]
pub struct CustomEmojiRepositorySqlx {
    pool: PgPool,
}

impl CustomEmojiRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomEmojiRepository for CustomEmojiRepositorySqlx {
    async fn create(&self, emoji: &CustomEmoji) -> Result<()> {
        sqlx::query!(
            "INSERT INTO custom_emoji (id, organization_id, shortcode, file_id, created_by, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (organization_id, shortcode) DO NOTHING",
            emoji.id.as_uuid(),
            emoji.organization_id.as_uuid(),
            emoji.shortcode,
            emoji.file_id.as_uuid(),
            emoji.created_by.as_uuid(),
            emoji.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: CustomEmojiId) -> Result<Option<CustomEmoji>> {
        let row = sqlx::query_as!(
            EmojiRow,
            "SELECT id, organization_id, shortcode, file_id, created_by, created_at FROM custom_emoji WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_emoji))
    }

    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomEmoji>> {
        let rows = sqlx::query_as!(
            EmojiRow,
            "SELECT id, organization_id, shortcode, file_id, created_by, created_at FROM custom_emoji WHERE organization_id = $1 ORDER BY shortcode",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_emoji).collect())
    }

    async fn delete(&self, id: CustomEmojiId) -> Result<Option<()>> {
        let result = sqlx::query!("DELETE FROM custom_emoji WHERE id = $1", id.as_uuid())
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
struct EmojiRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    shortcode: String,
    file_id: uuid::Uuid,
    created_by: uuid::Uuid,
    created_at: time::OffsetDateTime,
}

fn into_emoji(row: EmojiRow) -> CustomEmoji {
    CustomEmoji {
        id: CustomEmojiId::from_uuid(row.id),
        organization_id: OrganizationId::from_uuid(row.organization_id),
        shortcode: row.shortcode,
        file_id: ruckchat_id::FileId::from_uuid(row.file_id),
        created_by: ruckchat_id::UserId::from_uuid(row.created_by),
        created_at: row.created_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("custom emoji".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
