//! SQLx implementation of [`UserRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{User, UserRepository};
use ruckchat_id::UserId;
use sqlx::PgPool;

/// SQLx-backed user repository.
#[derive(Debug, Clone)]
pub struct UserRepositorySqlx {
    pool: PgPool,
}

impl UserRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositorySqlx {
    async fn create(&self, user: &User) -> Result<()> {
        sqlx::query!(
            "INSERT INTO users (id, email, display_name, password_hash, avatar_url, deactivated_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (email) DO NOTHING",
            user.id.as_uuid(),
            user.email,
            user.display_name,
            user.password_hash,
            user.avatar_url,
            user.deactivated_at,
            user.created_at,
            user.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn by_id(&self, id: UserId) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            "SELECT id, email, display_name, password_hash, avatar_url, deactivated_at, created_at, updated_at FROM users WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_user))
    }

    async fn by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            "SELECT id, email, display_name, password_hash, avatar_url, deactivated_at, created_at, updated_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(row.map(into_user))
    }

    async fn update(&self, user: &User) -> Result<()> {
        let rows = sqlx::query!(
            "UPDATE users
             SET email = $2, display_name = $3, password_hash = $4, avatar_url = $5, deactivated_at = $6, updated_at = $7
             WHERE id = $1",
            user.id.as_uuid(),
            user.email,
            user.display_name,
            user.password_hash,
            user.avatar_url,
            user.deactivated_at,
            user.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        if rows.rows_affected() == 0 {
            return Err(ruckchat_common::Error::NotFound("user".into()));
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    display_name: String,
    password_hash: String,
    avatar_url: Option<String>,
    deactivated_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

fn into_user(row: UserRow) -> User {
    User {
        id: UserId::from_uuid(row.id),
        email: row.email,
        display_name: row.display_name,
        password_hash: row.password_hash,
        avatar_url: row.avatar_url,
        deactivated_at: row.deactivated_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("user".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
