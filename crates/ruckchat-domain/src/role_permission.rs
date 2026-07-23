//! Role-permission link aggregate.

use ruckchat_id::{OrganizationRoleId, PermissionId};
use serde::{Deserialize, Serialize};

/// Grants a permission to a custom organization role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationRolePermission {
    /// Role receiving the permission.
    pub role_id: OrganizationRoleId,
    /// Permission being granted.
    pub permission_id: PermissionId,
}

impl OrganizationRolePermission {
    /// Creates a new role-permission grant.
    #[must_use]
    pub fn new(role_id: OrganizationRoleId, permission_id: PermissionId) -> Self {
        Self {
            role_id,
            permission_id,
        }
    }
}
