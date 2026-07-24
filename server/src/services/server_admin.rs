//! Server-wide administrative service.

use crate::services::{
    audit::AuditService,
    auth::AuthService,
    dto::{CreateOrganizationRequest, Pagination},
};
use ruckchat_common::{Error, Result};
use ruckchat_domain::{
    Organization, OrganizationMembershipRepository, OrganizationRepository, OrganizationSettings,
    OrganizationSettingsRepository, ServerSettingsRepository, User, UserRepository,
};
use ruckchat_id::{OrganizationId, UserId};
use std::sync::Arc;

/// Dependencies required by [`ServerAdminService`].
#[derive(Clone)]
pub struct ServerAdminServiceDeps {
    /// User repository.
    pub users: Arc<dyn UserRepository + Send + Sync>,
    /// Organization repository.
    pub organizations: Arc<dyn OrganizationRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn OrganizationMembershipRepository + Send + Sync>,
    /// Organization settings repository.
    pub organization_settings: Arc<dyn OrganizationSettingsRepository + Send + Sync>,
    /// Server settings repository.
    pub server_settings: Arc<dyn ServerSettingsRepository + Send + Sync>,
    /// Authentication service, used for impersonation sessions.
    pub auth: AuthService,
    /// Audit service.
    pub audit: AuditService,
}

/// Server-wide administration operations.
#[derive(Clone)]
pub struct ServerAdminService {
    deps: ServerAdminServiceDeps,
}

impl ServerAdminService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: ServerAdminServiceDeps) -> Self {
        Self { deps }
    }

    /// Lists all organizations in the server.
    pub async fn list_organizations(&self, caller_id: UserId) -> Result<Vec<Organization>> {
        self.require_server_admin(caller_id).await?;
        self.deps.organizations.list_all().await
    }

    /// Creates an organization without requiring the caller to join it.
    pub async fn create_organization(
        &self,
        caller_id: UserId,
        request: CreateOrganizationRequest,
    ) -> Result<Organization> {
        self.require_server_admin(caller_id).await?;
        if self
            .deps
            .organizations
            .by_slug(&request.slug)
            .await?
            .is_some()
        {
            return Err(Error::Conflict("organization slug already exists".into()));
        }
        let organization = Organization::new(request.name, request.slug, caller_id)?;
        self.deps.organizations.create(&organization).await?;
        let settings = OrganizationSettings::new(organization.id);
        self.deps.organization_settings.create(&settings).await?;
        self.audit(
            caller_id,
            None,
            "organization.created",
            "organization",
            Some(organization.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(organization)
    }

    /// Renames an organization.
    pub async fn rename_organization(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        name: String,
    ) -> Result<Organization> {
        self.require_server_admin(caller_id).await?;
        let mut organization = self
            .deps
            .organizations
            .by_id(organization_id)
            .await?
            .ok_or_else(|| Error::NotFound("organization".into()))?;
        organization.set_name(name)?;
        self.deps.organizations.update(&organization).await?;
        self.audit(
            caller_id,
            None,
            "organization.renamed",
            "organization",
            Some(organization.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(organization)
    }

    /// Deletes an organization and all its data.
    pub async fn delete_organization(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<()> {
        self.require_server_admin(caller_id).await?;
        self.deps
            .organizations
            .delete(organization_id)
            .await?
            .ok_or_else(|| Error::NotFound("organization".into()))?;
        self.audit(
            caller_id,
            None,
            "organization.deleted",
            "organization",
            Some(organization_id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Creates a new user account without an organization.
    pub async fn create_user(
        &self,
        caller_id: UserId,
        email: String,
        display_name: String,
        password: Option<String>,
    ) -> Result<(User, String)> {
        self.require_server_admin(caller_id).await?;
        if self.deps.users.by_email(&email).await?.is_some() {
            return Err(Error::Conflict("email already in use".into()));
        }
        let raw_password = password.unwrap_or_else(generate_temporary_password);
        let password_hash = crate::services::auth::hash_password(&raw_password)
            .map_err(|_| Error::Internal("password hash failed".into()))?;
        let user = User::new(email, display_name, password_hash)?;
        self.deps.users.create(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.created",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok((user, raw_password))
    }

    /// Lists all users with pagination.
    pub async fn list_users(&self, caller_id: UserId, pagination: Pagination) -> Result<Vec<User>> {
        self.require_server_admin(caller_id).await?;
        let pagination = pagination.normalized();
        self.deps
            .users
            .list_all(pagination.limit, pagination.offset)
            .await
    }

    /// Loads a user by id.
    pub async fn get_user(&self, caller_id: UserId, user_id: UserId) -> Result<User> {
        self.require_server_admin(caller_id).await?;
        self.deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))
    }

    /// Updates any user's profile.
    pub async fn update_user(
        &self,
        caller_id: UserId,
        user_id: UserId,
        display_name: Option<String>,
        avatar_url: Option<String>,
        email: Option<String>,
    ) -> Result<User> {
        self.require_server_admin(caller_id).await?;
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        if let Some(display_name) = display_name {
            user.set_display_name(display_name)?;
        }
        if avatar_url.is_some() {
            user.set_avatar_url(avatar_url);
        }
        if let Some(email) = email {
            if let Some(existing) = self.deps.users.by_email(&email).await?
                && existing.id != user_id
            {
                return Err(Error::Conflict("email already in use".into()));
            }
            user.set_email(email)?;
        }
        self.deps.users.update(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.updated",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(user)
    }

    /// Resets a user's password to a server-generated value.
    pub async fn reset_password(&self, caller_id: UserId, user_id: UserId) -> Result<String> {
        self.require_server_admin(caller_id).await?;
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        let new_password = generate_temporary_password();
        let hash = crate::services::auth::hash_password(&new_password)
            .map_err(|_| Error::Internal("password hash failed".into()))?;
        user.password_hash = hash;
        user.updated_at = ruckchat_common::time::OffsetDateTime::now_utc();
        self.deps.users.update(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.password_reset",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(new_password)
    }

    /// Promotes a user to server administrator.
    pub async fn promote_user(&self, caller_id: UserId, user_id: UserId) -> Result<User> {
        self.require_server_admin(caller_id).await?;
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        user.set_server_admin(true);
        self.deps.users.update(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.promoted",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(user)
    }

    /// Demotes a user from server administrator.
    pub async fn demote_user(&self, caller_id: UserId, user_id: UserId) -> Result<User> {
        self.require_server_admin(caller_id).await?;
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        if !user.is_server_admin {
            return Err(Error::Validation {
                message: "user is not a server admin".into(),
            });
        }
        let admin_count = self.deps.users.count_admins().await?;
        if admin_count <= 1 {
            return Err(Error::Validation {
                message: "cannot demote the last server admin".into(),
            });
        }
        user.set_server_admin(false);
        self.deps.users.update(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.demoted",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(user)
    }

    /// Lists current server administrators.
    pub async fn list_server_admins(&self, caller_id: UserId) -> Result<Vec<User>> {
        self.require_server_admin(caller_id).await?;
        self.deps.users.list_admins().await
    }

    /// Deactivates a user account.
    pub async fn deactivate_user(&self, caller_id: UserId, user_id: UserId) -> Result<User> {
        self.require_server_admin(caller_id).await?;
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        user.deactivate();
        self.deps.users.update(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.deactivated",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(user)
    }

    /// Reactivates a previously deactivated user account.
    pub async fn reactivate_user(&self, caller_id: UserId, user_id: UserId) -> Result<User> {
        self.require_server_admin(caller_id).await?;
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        user.reactivate();
        self.deps.users.update(&user).await?;
        self.audit(
            caller_id,
            None,
            "user.reactivated",
            "user",
            Some(user.id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(user)
    }

    /// Starts an impersonation session for a target user.
    pub async fn impersonate(&self, caller_id: UserId, target_user_id: UserId) -> Result<String> {
        self.require_server_admin(caller_id).await?;
        let _target = self
            .deps
            .users
            .by_id(target_user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;
        let token = self
            .deps
            .auth
            .create_impersonation_session(target_user_id, caller_id)
            .await
            .map_err(map_server_err)?;
        self.audit(
            caller_id,
            Some(target_user_id),
            "impersonation.started",
            "user",
            Some(target_user_id.as_uuid()),
            None,
            None,
        )
        .await?;
        Ok(token)
    }

    /// Ends the current impersonation session.
    pub async fn end_impersonate(&self, caller_id: UserId, token: &str) -> Result<()> {
        self.require_server_admin(caller_id).await?;
        self.deps
            .auth
            .end_impersonation_session(token)
            .await
            .map_err(map_server_err)?;
        self.audit(
            caller_id,
            None,
            "impersonation.ended",
            "session",
            None,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    /// Verifies the caller is a server administrator.
    pub async fn require_server_admin(&self, caller_id: UserId) -> Result<()> {
        let caller = self
            .deps
            .users
            .by_id(caller_id)
            .await?
            .ok_or_else(|| Error::Forbidden("server admin access required".into()))?;
        if !caller.is_server_admin {
            return Err(Error::Forbidden("server admin access required".into()));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn audit(
        &self,
        actor_id: UserId,
        impersonated_user_id: Option<UserId>,
        action: &str,
        resource_type: &str,
        resource_id: Option<uuid::Uuid>,
        metadata: Option<serde_json::Value>,
        ip_address: Option<&str>,
    ) -> Result<()> {
        self.deps
            .audit
            .record(
                actor_id,
                impersonated_user_id,
                None,
                action,
                resource_type,
                resource_id,
                metadata,
                ip_address,
            )
            .await
    }
}

fn generate_temporary_password() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn map_server_err(err: crate::Error) -> ruckchat_common::Error {
    match err {
        crate::Error::Domain(e) => e,
        crate::Error::PasswordHash => {
            ruckchat_common::Error::Internal("password operation failed".into())
        }
        crate::Error::TokenGeneration => {
            ruckchat_common::Error::Internal("token generation failed".into())
        }
    }
}
