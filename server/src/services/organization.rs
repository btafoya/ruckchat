//! Organization service.

use crate::services::authorization::{AuthorizationService, Permission};
use crate::services::dto::{ChangeRoleRequest, CreateOrganizationRequest, InviteMemberRequest};
use ruckchat_common::Error;
use ruckchat_domain::{
    Organization, OrganizationMembership, OrganizationRepository, OrganizationSettings, Role,
    UserRepository,
};
use ruckchat_id::{OrganizationId, UserId};
use std::sync::Arc;

/// Dependencies required by [`OrganizationService`].
#[derive(Clone)]
pub struct OrganizationServiceDeps {
    /// Organization repository.
    pub organizations: Arc<dyn OrganizationRepository + Send + Sync>,
    /// User repository.
    pub users: Arc<dyn UserRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync>,
    /// Organization settings repository.
    pub settings: Arc<dyn ruckchat_domain::OrganizationSettingsRepository + Send + Sync>,
    /// Authorization service.
    pub authorization: AuthorizationService,
}

/// Organization and membership operations.
#[derive(Clone)]
pub struct OrganizationService {
    deps: OrganizationServiceDeps,
}

impl OrganizationService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: OrganizationServiceDeps) -> Self {
        Self { deps }
    }

    /// Creates an organization and makes the caller the owner.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] for invalid input or [`Error::Conflict`]
    /// when the slug exists.
    pub async fn create_organization(
        &self,
        caller_id: UserId,
        request: CreateOrganizationRequest,
    ) -> ruckchat_common::Result<Organization> {
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

        let membership = OrganizationMembership::new(caller_id, organization.id, Role::Owner)?;
        self.deps.memberships.create(&membership).await?;

        let settings = OrganizationSettings::new(organization.id);
        self.deps.settings.create(&settings).await?;

        Ok(organization)
    }

    /// Lists organizations the caller belongs to.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Internal`] for database failures.
    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> ruckchat_common::Result<Vec<Organization>> {
        self.deps.organizations.list_for_user(user_id).await
    }

    /// Loads an organization by slug.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the organization does not exist.
    pub async fn get_by_slug(&self, slug: &str) -> ruckchat_common::Result<Organization> {
        self.deps
            .organizations
            .by_slug(slug)
            .await?
            .ok_or_else(|| Error::NotFound("organization".into()))
    }

    /// Invites an existing user to the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission,
    /// [`Error::NotFound`] when the target user does not exist, or
    /// [`Error::Conflict`] when the user is already a member.
    pub async fn invite_member(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        request: InviteMemberRequest,
    ) -> ruckchat_common::Result<OrganizationMembership> {
        let caller_membership = self.require_membership(caller_id, organization_id).await?;
        self.deps
            .authorization
            .require_role_permission(caller_membership.role, Permission::ManageOrganization)?;

        let target_user = self
            .deps
            .users
            .by_email(&request.email)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;

        if self
            .deps
            .memberships
            .by_ids(target_user.id, organization_id)
            .await?
            .is_some()
        {
            return Err(Error::Conflict("user is already a member".into()));
        }

        let membership =
            OrganizationMembership::new(target_user.id, organization_id, request.role)?;
        self.deps.memberships.create(&membership).await?;
        Ok(membership)
    }

    /// Changes a member's role.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission or
    /// [`Error::NotFound`] when the target is not a member.
    pub async fn change_member_role(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        request: ChangeRoleRequest,
    ) -> ruckchat_common::Result<()> {
        let caller_membership = self.require_membership(caller_id, organization_id).await?;
        self.deps
            .authorization
            .require_role_permission(caller_membership.role, Permission::ManageOrganization)?;

        if self
            .deps
            .memberships
            .by_ids(request.user_id, organization_id)
            .await?
            .is_none()
        {
            return Err(Error::NotFound("organization membership".into()));
        }

        self.deps
            .memberships
            .update_role(request.user_id, organization_id, request.role)
            .await?;
        Ok(())
    }

    /// Removes a member from the organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission or
    /// [`Error::NotFound`] when the target is not a member.
    pub async fn remove_member(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> ruckchat_common::Result<()> {
        let caller_membership = self.require_membership(caller_id, organization_id).await?;
        self.deps
            .authorization
            .require_role_permission(caller_membership.role, Permission::ManageOrganization)?;

        if caller_id == user_id {
            return Err(Error::Forbidden("cannot remove yourself".into()));
        }

        self.deps
            .memberships
            .delete(user_id, organization_id)
            .await?;
        Ok(())
    }

    async fn require_membership(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<OrganizationMembership> {
        self.deps
            .memberships
            .by_ids(user_id, organization_id)
            .await?
            .ok_or_else(|| Error::Forbidden("must be an organization member".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::authorization::AuthorizationService;
    use crate::services::dto::{ChangeRoleRequest, CreateOrganizationRequest, InviteMemberRequest};
    use crate::testing::{
        MockOrganizationMembershipRepository, MockOrganizationRepository,
        MockOrganizationSettingsRepository, MockUserRepository,
    };
    use ruckchat_domain::{OrganizationMembership, Role, User};
    use ruckchat_id::{OrganizationId, UserId};
    use std::sync::Arc;

    fn service() -> OrganizationService {
        OrganizationService::new(OrganizationServiceDeps {
            organizations: Arc::new(MockOrganizationRepository::new()),
            users: Arc::new(MockUserRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
            settings: Arc::new(MockOrganizationSettingsRepository::new()),
            authorization: AuthorizationService::new(),
        })
    }

    async fn seed_user_and_org(svc: &OrganizationService) -> (UserId, OrganizationId) {
        let user = User::new("owner@example.com", "Owner", "hash").unwrap();
        svc.deps.users.create(&user).await.unwrap();
        let org = svc
            .create_organization(
                user.id,
                CreateOrganizationRequest {
                    name: "Acme".into(),
                    slug: "acme".into(),
                },
            )
            .await
            .unwrap();
        (user.id, org.id)
    }

    #[tokio::test]
    async fn create_organization_makes_owner() {
        let svc = service();
        let (owner_id, org_id) = seed_user_and_org(&svc).await;
        let membership = svc
            .deps
            .memberships
            .by_ids(owner_id, org_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(membership.role, Role::Owner);
    }

    #[tokio::test]
    async fn invite_member_requires_existing_user() {
        let svc = service();
        let (owner_id, org_id) = seed_user_and_org(&svc).await;
        let err = svc
            .invite_member(
                owner_id,
                org_id,
                InviteMemberRequest {
                    email: "nobody@example.com".into(),
                    role: Role::Member,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[tokio::test]
    async fn member_cannot_invite_others() {
        let svc = service();
        let (_owner_id, org_id) = seed_user_and_org(&svc).await;
        let member = User::new("member@example.com", "Member", "hash").unwrap();
        svc.deps.users.create(&member).await.unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(member.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .invite_member(
                member.id,
                org_id,
                InviteMemberRequest {
                    email: "other@example.com".into(),
                    role: Role::Member,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn owner_can_change_role() {
        let svc = service();
        let (owner_id, org_id) = seed_user_and_org(&svc).await;
        let member = User::new("member@example.com", "Member", "hash").unwrap();
        svc.deps.users.create(&member).await.unwrap();
        svc.invite_member(
            owner_id,
            org_id,
            InviteMemberRequest {
                email: "member@example.com".into(),
                role: Role::Member,
            },
        )
        .await
        .unwrap();

        svc.change_member_role(
            owner_id,
            org_id,
            ChangeRoleRequest {
                user_id: member.id,
                role: Role::Admin,
            },
        )
        .await
        .unwrap();

        let membership = svc
            .deps
            .memberships
            .by_ids(member.id, org_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(membership.role, Role::Admin);
    }
}
