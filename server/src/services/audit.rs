//! Audit log service.

use ruckchat_common::Result;
use ruckchat_domain::{AuditLogEntry, AuditLogRepository};
use ruckchat_id::{OrganizationId, UserId};
use std::sync::Arc;

/// Dependencies required by [`AuditService`].
#[derive(Clone)]
pub struct AuditServiceDeps {
    /// Audit log repository.
    pub audit_log: Arc<dyn AuditLogRepository + Send + Sync>,
}

/// Append-only audit log writer.
#[derive(Clone)]
pub struct AuditService {
    deps: AuditServiceDeps,
}

impl AuditService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: AuditServiceDeps) -> Self {
        Self { deps }
    }

    /// Records an audit log entry.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Internal`] for repository failures.
    #[allow(clippy::too_many_arguments)]
    pub async fn record(
        &self,
        actor_id: UserId,
        impersonated_user_id: Option<UserId>,
        organization_id: Option<OrganizationId>,
        action: &str,
        resource_type: &str,
        resource_id: Option<uuid::Uuid>,
        metadata: Option<serde_json::Value>,
        ip_address: Option<&str>,
    ) -> Result<()> {
        let entry = AuditLogEntry {
            id: uuid::Uuid::new_v4(),
            occurred_at: ruckchat_common::time::OffsetDateTime::now_utc(),
            actor_id,
            impersonated_user_id,
            organization_id,
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id,
            metadata,
            ip_address: ip_address.map(Into::into),
        };
        self.deps.audit_log.create(&entry).await
    }

    /// Queries audit log entries.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Internal`] for repository failures.
    #[allow(clippy::too_many_arguments)]
    pub async fn query(
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
        self.deps
            .audit_log
            .query(
                actor_id,
                organization_id,
                action,
                resource_type,
                from,
                to,
                limit,
                offset,
            )
            .await
    }
}
