//! User service.

use crate::services::dto::{Pagination, UpdateProfileRequest};
use ruckchat_common::Error;
use ruckchat_domain::{User, UserRepository};
use ruckchat_id::{OrganizationId, UserId};
use std::sync::Arc;

/// Dependencies required by [`UserService`].
#[derive(Clone)]
pub struct UserServiceDeps {
    /// User repository.
    pub users: Arc<dyn UserRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync>,
}

/// User profile and membership operations.
#[derive(Clone)]
pub struct UserService {
    deps: UserServiceDeps,
}

impl UserService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: UserServiceDeps) -> Self {
        Self { deps }
    }

    /// Loads a user profile by id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the user does not exist.
    pub async fn get_profile(&self,
        user_id: UserId,
    ) -> ruckchat_common::Result<User> {
        self.deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))
    }

    /// Updates the caller's profile.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the user does not exist or
    /// [`Error::Validation`] for invalid input.
    pub async fn update_profile(
        &self,
        user_id: UserId,
        request: UpdateProfileRequest,
    ) -> ruckchat_common::Result<User> {
        let mut user = self
            .deps
            .users
            .by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound("user".into()))?;

        if let Some(display_name) = request.display_name {
            user.set_display_name(display_name)?;
        }
        if request.avatar_url.is_some() {
            user.set_avatar_url(request.avatar_url);
        }

        self.deps.users.update(&user).await?;
        Ok(user)
    }

    /// Lists users in an organization. The caller must be a member.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not a member.
    pub async fn list_users_in_organization(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        pagination: Pagination,
    ) -> ruckchat_common::Result<Vec<User>> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?;
        if membership.is_none() {
            return Err(Error::Forbidden(
                "must be an organization member to list users".into(),
            ));
        }

        let members = self
            .deps
            .memberships
            .list_by_organization(organization_id)
            .await?;

        let pagination = pagination.normalized();
        let mut users = Vec::with_capacity(members.len().min(pagination.limit as usize));
        for (idx, membership) in members.into_iter().enumerate() {
            let offset = pagination.offset as usize;
            if idx < offset {
                continue;
            }
            if users.len() >= pagination.limit as usize {
                break;
            }
            if let Some(user) = self.deps.users.by_id(membership.user_id).await? {
                users.push(user);
            }
        }

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dto::UpdateProfileRequest;
    use crate::testing::{
        MockOrganizationMembershipRepository, MockUserRepository,
    };
    use ruckchat_domain::{OrganizationMembership, Role, User};
    use ruckchat_id::OrganizationId;
    use std::sync::Arc;

    fn service() -> UserService {
        UserService::new(UserServiceDeps {
            users: Arc::new(MockUserRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
        })
    }

    #[tokio::test]
    async fn get_profile_returns_user() {
        let svc = service();
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        svc.deps.users.create(&user).await.unwrap();

        let found = svc.get_profile(user.id).await.unwrap();
        assert_eq!(found.email, "alice@example.com");
    }

    #[tokio::test]
    async fn update_profile_changes_display_name() {
        let svc = service();
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        svc.deps.users.create(&user).await.unwrap();

        let updated = svc
            .update_profile(
                user.id,
                UpdateProfileRequest {
                    display_name: Some("Alice Updated".into()),
                    avatar_url: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.display_name, "Alice Updated");
    }

    #[tokio::test]
    async fn list_users_requires_membership() {
        let svc = service();
        let org_id = OrganizationId::new();
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        svc.deps.users.create(&user).await.unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let outsider = User::new("outsider@example.com", "Outsider", "hash").unwrap();
        svc.deps.users.create(&outsider).await.unwrap();

        let err = svc
            .list_users_in_organization(
                outsider.id,
                org_id,
                Pagination::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
