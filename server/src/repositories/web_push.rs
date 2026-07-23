//! SQLx implementation of [`WebPushSubscriptionRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{WebPushSubscription, WebPushSubscriptionRepository};
use ruckchat_id::UserId;
use sqlx::PgPool;
use time::OffsetDateTime;

/// SQLx-backed Web Push subscription repository.
#[derive(Debug, Clone)]
pub struct WebPushSubscriptionRepositorySqlx {
    pool: PgPool,
}

impl WebPushSubscriptionRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebPushSubscriptionRepository for WebPushSubscriptionRepositorySqlx {
    async fn upsert(&self, subscription: &WebPushSubscription) -> Result<()> {
        sqlx::query!(
            "INSERT INTO web_push_subscriptions (id, user_id, endpoint, p256dh, auth, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)
             ON CONFLICT (user_id, endpoint)
             DO UPDATE SET p256dh = EXCLUDED.p256dh,
                           auth = EXCLUDED.auth,
                           updated_at = EXCLUDED.updated_at",
            subscription.id,
            subscription.user_id.as_uuid(),
            subscription.endpoint,
            subscription.p256dh,
            subscription.auth,
            subscription.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<WebPushSubscription>> {
        let rows = sqlx::query_as!(
            SubscriptionRow,
            "SELECT id, user_id, endpoint, p256dh, auth, created_at, updated_at
             FROM web_push_subscriptions
             WHERE user_id = $1
             ORDER BY created_at",
            user_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_subscription).collect())
    }

    async fn delete_by_endpoint(&self, user_id: UserId, endpoint: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM web_push_subscriptions WHERE user_id = $1 AND endpoint = $2",
            user_id.as_uuid(),
            endpoint,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    endpoint: String,
    p256dh: String,
    auth: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn into_subscription(row: SubscriptionRow) -> WebPushSubscription {
    WebPushSubscription {
        id: row.id,
        user_id: UserId::from_uuid(row.user_id),
        endpoint: row.endpoint,
        p256dh: row.p256dh,
        auth: row.auth,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => {
            ruckchat_common::Error::NotFound("web push subscription".into())
        }
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
