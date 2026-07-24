//! SQLx implementation of [`SessionRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{Session, SessionRepository};
use ruckchat_id::{SessionId, UserId};
use sqlx::PgPool;

/// SQLx-backed session repository.
#[derive(Debug, Clone)]
pub struct SessionRepositorySqlx {
    pool: PgPool,
}

impl SessionRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for SessionRepositorySqlx {
    async fn create(&self, session: &Session) -> Result<()> {
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_hash, expires_at, created_at, ip_address, user_agent, impersonated_by)
             VALUES ($1, $2, $3, $4, $5, $6::inet, $7, $8)
             ON CONFLICT (token_hash) DO NOTHING",
        )
        .bind(session.id.as_uuid())
        .bind(session.user_id.as_uuid())
        .bind(&session.token_hash)
        .bind(session.expires_at)
        .bind(session.created_at)
        .bind(session.ip_address.as_deref())
        .bind(session.user_agent.as_deref())
        .bind(session.impersonated_by.map(|id| id.as_uuid()))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: SessionId) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, user_id, token_hash, expires_at, created_at, ip_address::text AS ip_address, user_agent, impersonated_by FROM sessions WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_session))
    }

    async fn by_token_hash(&self, token_hash: &str) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, user_id, token_hash, expires_at, created_at, ip_address::text AS ip_address, user_agent, impersonated_by FROM sessions WHERE token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_session))
    }

    async fn delete_expired(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(result.rows_affected())
    }

    async fn delete_by_token_hash(&self, token_hash: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(ruckchat_common::Error::NotFound("session".into()));
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    token_hash: String,
    expires_at: time::OffsetDateTime,
    created_at: time::OffsetDateTime,
    ip_address: Option<String>,
    user_agent: Option<String>,
    impersonated_by: Option<uuid::Uuid>,
}

fn into_session(row: SessionRow) -> Session {
    Session {
        id: SessionId::from_uuid(row.id),
        user_id: UserId::from_uuid(row.user_id),
        token_hash: row.token_hash,
        expires_at: row.expires_at,
        created_at: row.created_at,
        ip_address: row.ip_address,
        user_agent: row.user_agent,
        impersonated_by: row.impersonated_by.map(UserId::from_uuid),
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("session".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
