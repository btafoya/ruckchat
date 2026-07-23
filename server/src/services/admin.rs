//! Administrative service for organization-level imports and metadata.
//!
//! Operations in this module are restricted to organization owners and admins.

use crate::migrate::{self, ImportCounts, MigrateError, MigrationData};
use ruckchat_common::{Error, Result};
use ruckchat_domain::{
    CustomEmoji, CustomEmojiRepository, Organization, OrganizationMembership,
    OrganizationMembershipRepository, OrganizationRepository, OrganizationRole,
    OrganizationRoleRepository, Permission, PermissionRepository, Team, TeamRepository,
};
use ruckchat_id::{FileId, OrganizationId, UserId};
use sqlx::PgPool;
use std::sync::Arc;

/// Dependencies required by [`AdminService`].
#[derive(Clone)]
pub struct AdminServiceDeps {
    /// PostgreSQL connection pool used for snapshot imports.
    pub pool: PgPool,
    /// Organization repository.
    pub organizations: Arc<dyn OrganizationRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn OrganizationMembershipRepository + Send + Sync>,
    /// Custom role repository.
    pub roles: Arc<dyn OrganizationRoleRepository + Send + Sync>,
    /// Permission repository.
    pub permissions: Arc<dyn PermissionRepository + Send + Sync>,
    /// Custom emoji repository.
    pub emoji: Arc<dyn CustomEmojiRepository + Send + Sync>,
    /// Team repository.
    pub teams: Arc<dyn TeamRepository + Send + Sync>,
    /// File repository, used to validate emoji file references.
    pub files: Arc<dyn ruckchat_domain::FileRepository + Send + Sync>,
}

/// Organization administration operations.
#[derive(Clone)]
pub struct AdminService {
    deps: AdminServiceDeps,
}

impl AdminService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: AdminServiceDeps) -> Self {
        Self { deps }
    }

    /// Imports a migration snapshot into the target organization.
    ///
    /// The caller must be an owner or admin of the organization. The snapshot
    /// is validated to ensure it does not reference organizations other than the
    /// target, then written idempotently through the migration subsystem.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an admin,
    /// [`Error::NotFound`] when the organization does not exist, or
    /// [`Error::Validation`] when the snapshot is inconsistent or targets a
    /// different organization.
    pub async fn import_snapshot(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        data: &MigrationData,
        dry_run: bool,
    ) -> Result<ImportCounts> {
        self.require_admin(caller_id, organization_id).await?;
        self.ensure_organization_exists(organization_id).await?;
        validate_target_organization(data, organization_id)?;

        migrate::import(&self.deps.pool, data, dry_run)
            .await
            .map_err(|err| Error::from(MigrateErrorWrapper(err)))
    }

    /// Lists custom roles defined in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an admin.
    pub async fn list_roles(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationRole>> {
        self.require_admin(caller_id, organization_id).await?;
        self.deps.roles.list_by_organization(organization_id).await
    }

    /// Creates a custom role in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] for non-admins or [`Error::Validation`] for
    /// invalid input.
    pub async fn create_role(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        name: String,
        description: Option<String>,
    ) -> Result<OrganizationRole> {
        self.require_admin(caller_id, organization_id).await?;
        let role = OrganizationRole::new(organization_id, name, description)?;
        self.deps.roles.create(&role).await?;
        Ok(role)
    }

    /// Lists permissions defined in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an admin.
    pub async fn list_permissions(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<Permission>> {
        self.require_admin(caller_id, organization_id).await?;
        self.deps
            .permissions
            .list_by_organization(organization_id)
            .await
    }

    /// Creates a permission in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] for non-admins or [`Error::Validation`] for
    /// invalid input.
    pub async fn create_permission(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        key: String,
        description: Option<String>,
    ) -> Result<Permission> {
        self.require_admin(caller_id, organization_id).await?;
        let permission = Permission::new(organization_id, key, description)?;
        self.deps.permissions.create(&permission).await?;
        Ok(permission)
    }

    /// Lists custom emoji defined in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an admin.
    pub async fn list_emoji(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomEmoji>> {
        self.require_admin(caller_id, organization_id).await?;
        self.deps.emoji.list_by_organization(organization_id).await
    }

    /// Creates a custom emoji in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] for non-admins, [`Error::NotFound`] when the
    /// referenced file does not exist, or [`Error::Validation`] for invalid input.
    pub async fn create_emoji(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        shortcode: String,
        file_id: FileId,
    ) -> Result<CustomEmoji> {
        self.require_admin(caller_id, organization_id).await?;

        let file = self
            .deps
            .files
            .by_id(file_id)
            .await?
            .ok_or_else(|| Error::NotFound("file".into()))?;
        if file.organization_id != organization_id {
            return Err(Error::Forbidden(
                "file does not belong to this organization".into(),
            ));
        }

        let emoji = CustomEmoji::new(organization_id, shortcode, file_id, caller_id)?;
        self.deps.emoji.create(&emoji).await?;
        Ok(emoji)
    }

    /// Lists teams defined in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an admin.
    pub async fn list_teams(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<Team>> {
        self.require_admin(caller_id, organization_id).await?;
        self.deps.teams.list_by_organization(organization_id).await
    }

    /// Creates a team in the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] for non-admins or [`Error::Validation`] for
    /// invalid input.
    pub async fn create_team(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        name: String,
        description: Option<String>,
    ) -> Result<Team> {
        self.require_admin(caller_id, organization_id).await?;
        let team = Team::new(organization_id, name, description, caller_id)?;
        self.deps.teams.create(&team).await?;
        Ok(team)
    }

    async fn require_admin(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<OrganizationMembership> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?
            .ok_or_else(|| Error::Forbidden("must be an organization member".into()))?;

        if !membership.role.is_manager() {
            return Err(Error::Forbidden("admin access required".into()));
        }

        Ok(membership)
    }

    async fn ensure_organization_exists(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Organization> {
        self.deps
            .organizations
            .by_id(organization_id)
            .await?
            .ok_or_else(|| Error::NotFound("organization".into()))
    }
}

/// Converts a migration error into a domain error.
struct MigrateErrorWrapper(MigrateError);

impl From<MigrateErrorWrapper> for Error {
    fn from(err: MigrateErrorWrapper) -> Self {
        match err.0 {
            MigrateError::Validation(_) | MigrateError::UnsupportedVersion(_) => {
                Error::Validation {
                    message: err.0.to_string(),
                }
            }
            _ => Error::Internal(err.0.to_string()),
        }
    }
}

/// Validates that a snapshot only references the target organization.
fn validate_target_organization(
    data: &MigrationData,
    organization_id: OrganizationId,
) -> Result<()> {
    let target_uuid = organization_id.as_uuid();

    if data.organizations.len() > 1 {
        return Err(Error::validation(
            "snapshot must reference exactly one organization",
        ));
    }

    if let Some(org) = data.organizations.first() {
        if org.id.as_uuid() != target_uuid {
            return Err(Error::validation(
                "snapshot organization does not match target",
            ));
        }
    } else if !data.organization_roles.is_empty()
        || !data.permissions.is_empty()
        || !data.custom_emoji.is_empty()
        || !data.teams.is_empty()
        || !data.organization_settings.is_empty()
        || !data.channels.is_empty()
        || !data.files.is_empty()
        || !data.organization_memberships.is_empty()
        || !data.direct_message_conversations.is_empty()
    {
        return Err(Error::validation(
            "snapshot contains organization-scoped data but no organization",
        ));
    }

    for role in &data.organization_roles {
        if role.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "role {} belongs to a different organization",
                role.id
            )));
        }
    }

    for permission in &data.permissions {
        if permission.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "permission {} belongs to a different organization",
                permission.id
            )));
        }
    }

    for emoji in &data.custom_emoji {
        if emoji.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "custom emoji {} belongs to a different organization",
                emoji.id
            )));
        }
    }

    for team in &data.teams {
        if team.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "team {} belongs to a different organization",
                team.id
            )));
        }
    }

    for settings in &data.organization_settings {
        if settings.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "settings for {} belong to a different organization",
                settings.organization_id
            )));
        }
    }

    for channel in &data.channels {
        if channel.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "channel {} belongs to a different organization",
                channel.id
            )));
        }
    }

    for file in &data.files {
        if file.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "file {} belongs to a different organization",
                file.id
            )));
        }
    }

    for membership in &data.organization_memberships {
        if membership.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "organization membership for user {} belongs to a different organization",
                membership.user_id
            )));
        }
    }

    for conversation in &data.direct_message_conversations {
        if conversation.organization_id.as_uuid() != target_uuid {
            return Err(Error::validation(format!(
                "direct message conversation {} belongs to a different organization",
                conversation.id
            )));
        }
    }

    Ok(())
}
