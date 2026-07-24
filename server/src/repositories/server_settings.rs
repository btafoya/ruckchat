//! SQLx implementation of [`ServerSettingsRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{ServerSettings, ServerSettingsRepository};
use ruckchat_id::UserId;
use sqlx::PgPool;

/// SQLx-backed server settings repository.
#[derive(Debug, Clone)]
pub struct ServerSettingsRepositorySqlx {
    pool: PgPool,
}

impl ServerSettingsRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServerSettingsRepository for ServerSettingsRepositorySqlx {
    async fn load(&self) -> Result<ServerSettings> {
        let rows = sqlx::query_as!(SettingRow, "SELECT key, value FROM server_settings")
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let mut settings = ServerSettings::defaults();
        for row in rows {
            match row.key.as_str() {
                "maintenance_mode_enabled" => {
                    settings.maintenance_mode_enabled = row.value.parse().unwrap_or(false);
                }
                "default_max_file_size_bytes" => {
                    settings.default_max_file_size_bytes =
                        row.value.parse().unwrap_or(25 * 1024 * 1024);
                }
                "default_storage_quota_bytes" => {
                    settings.default_storage_quota_bytes =
                        row.value.parse().unwrap_or(10 * 1024 * 1024 * 1024);
                }
                "allowed_signup_domains" => {
                    settings.allowed_signup_domains =
                        serde_json::from_str(&row.value).unwrap_or_default();
                }
                "allow_registration" => {
                    settings.allow_registration = row.value.parse().unwrap_or(true);
                }
                "spelling_enabled" => {
                    settings.spelling_enabled = row.value.parse().unwrap_or(true);
                }
                "spelling_default_language" => {
                    settings.spelling_default_language = row.value.clone();
                }
                _ => {}
            }
        }
        Ok(settings)
    }

    async fn save(&self, settings: &ServerSettings, updated_by: UserId) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        let pairs = [
            (
                "maintenance_mode_enabled",
                settings.maintenance_mode_enabled.to_string(),
            ),
            (
                "default_max_file_size_bytes",
                settings.default_max_file_size_bytes.to_string(),
            ),
            (
                "default_storage_quota_bytes",
                settings.default_storage_quota_bytes.to_string(),
            ),
            (
                "allowed_signup_domains",
                serde_json::to_string(&settings.allowed_signup_domains)
                    .unwrap_or_else(|_| "[]".into()),
            ),
            (
                "allow_registration",
                settings.allow_registration.to_string(),
            ),
            ("spelling_enabled", settings.spelling_enabled.to_string()),
            (
                "spelling_default_language",
                settings.spelling_default_language.clone(),
            ),
        ];

        for (key, value) in pairs {
            sqlx::query!(
                "INSERT INTO server_settings (key, value, updated_at, updated_by)
                 VALUES ($1, $2, NOW(), $3)
                 ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW(), updated_by = $3",
                key,
                value,
                updated_by.as_uuid()
            )
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;
        }

        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SettingRow {
    key: String,
    value: String,
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    match err {
        sqlx::Error::RowNotFound => ruckchat_common::Error::NotFound("server setting".into()),
        _ => ruckchat_common::Error::Internal(err.to_string()),
    }
}
