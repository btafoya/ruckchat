//! SQLx implementation of [`AuditLogRepository`].

use async_trait::async_trait;
use ruckchat_common::Result;
use ruckchat_domain::{AuditLogEntry, AuditLogRepository};
use ruckchat_id::{OrganizationId, UserId};
use sqlx::PgPool;

/// SQLx-backed audit log repository.
#[derive(Debug, Clone)]
pub struct AuditLogRepositorySqlx {
    pool: PgPool,
}

impl AuditLogRepositorySqlx {
    /// Creates a repository backed by the supplied connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepository for AuditLogRepositorySqlx {
    async fn create(&self, entry: &AuditLogEntry) -> Result<()> {
        let ip_network = entry
            .ip_address
            .as_deref()
            .and_then(|ip| ip.parse::<ipnetwork::IpNetwork>().ok());
        sqlx::query!(
            "INSERT INTO audit_log (id, occurred_at, actor_id, impersonated_user_id, organization_id, action, resource_type, resource_id, metadata, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::inet)",
            entry.id,
            entry.occurred_at,
            entry.actor_id.as_uuid(),
            entry.impersonated_user_id.map(|id| id.as_uuid()),
            entry.organization_id.map(|id| id.as_uuid()),
            entry.action,
            entry.resource_type,
            entry.resource_id,
            entry.metadata,
            ip_network
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn query(
        &self,
        actor_id: Option<UserId>,
        organization_id: Option<OrganizationId>,
        action: Option<&str>,
        resource_type: Option<&str>,
        from: Option<ruckchat_common::time::OffsetDateTime>,
        to: Option<ruckchat_common::time::OffsetDateTime>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntry>> {
        let rows = sqlx::query_as!(
            AuditLogRow,
            "SELECT id, occurred_at, actor_id, impersonated_user_id, organization_id, action, resource_type, resource_id, metadata, ip_address::text AS ip_address
             FROM audit_log
             WHERE ($1::uuid IS NULL OR actor_id = $1)
               AND ($2::uuid IS NULL OR organization_id = $2)
               AND ($3::varchar IS NULL OR action = $3)
               AND ($4::varchar IS NULL OR resource_type = $4)
               AND ($5::timestamptz IS NULL OR occurred_at >= $5)
               AND ($6::timestamptz IS NULL OR occurred_at <= $6)
             ORDER BY occurred_at DESC
             LIMIT $7 OFFSET $8",
            actor_id.map(|id| id.as_uuid()),
            organization_id.map(|id| id.as_uuid()),
            action,
            resource_type,
            from,
            to,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(rows.into_iter().map(into_entry).collect())
    }
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: uuid::Uuid,
    occurred_at: time::OffsetDateTime,
    actor_id: uuid::Uuid,
    impersonated_user_id: Option<uuid::Uuid>,
    organization_id: Option<uuid::Uuid>,
    action: String,
    resource_type: String,
    resource_id: Option<uuid::Uuid>,
    metadata: Option<serde_json::Value>,
    ip_address: Option<String>,
}

fn into_entry(row: AuditLogRow) -> AuditLogEntry {
    AuditLogEntry {
        id: row.id,
        occurred_at: row.occurred_at,
        actor_id: UserId::from_uuid(row.actor_id),
        impersonated_user_id: row.impersonated_user_id.map(UserId::from_uuid),
        organization_id: row.organization_id.map(OrganizationId::from_uuid),
        action: row.action,
        resource_type: row.resource_type,
        resource_id: row.resource_id,
        metadata: row.metadata,
        ip_address: row.ip_address,
    }
}

fn map_sqlx_err(err: sqlx::Error) -> ruckchat_common::Error {
    ruckchat_common::Error::Internal(err.to_string())
}
