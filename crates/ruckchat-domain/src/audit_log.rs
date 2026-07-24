//! Audit log aggregate.

use ruckchat_id::{OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// An append-only audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Internal entry identifier.
    pub id: uuid::Uuid,
    /// Timestamp when the action occurred.
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: ruckchat_common::time::OffsetDateTime,
    /// User who performed the action.
    pub actor_id: UserId,
    /// User being impersonated, if any.
    pub impersonated_user_id: Option<UserId>,
    /// Organization the action targeted, if any.
    pub organization_id: Option<OrganizationId>,
    /// Action type.
    pub action: String,
    /// Type of resource affected.
    pub resource_type: String,
    /// Identifier of the affected resource, if any.
    pub resource_id: Option<uuid::Uuid>,
    /// Additional action-specific metadata.
    pub metadata: Option<serde_json::Value>,
    /// Client IP address, if available.
    pub ip_address: Option<String>,
}
