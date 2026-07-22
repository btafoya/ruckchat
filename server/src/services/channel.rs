//! Channel service.

use crate::services::authorization::{AuthorizationService, Permission};
use crate::services::dto::{CreateChannelRequest, UpdateChannelRequest};
use ruckchat_common::Error;
use ruckchat_domain::{
    Channel, ChannelMembership, ChannelRepository, OrganizationMembershipRepository,
};
use ruckchat_id::{ChannelId, OrganizationId, UserId};
use std::sync::Arc;

/// Dependencies required by [`ChannelService`].
#[derive(Clone)]
pub struct ChannelServiceDeps {
    /// Channel repository.
    pub channels: Arc<dyn ChannelRepository + Send + Sync>,
    /// Channel membership repository.
    pub channel_memberships: Arc<dyn ruckchat_domain::ChannelMembershipRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn OrganizationMembershipRepository + Send + Sync>,
    /// Authorization service.
    pub authorization: AuthorizationService,
}

/// Channel and membership operations.
#[derive(Clone)]
pub struct ChannelService {
    deps: ChannelServiceDeps,
}

impl ChannelService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: ChannelServiceDeps) -> Self {
        Self { deps }
    }

    /// Creates a channel within an organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization
    /// member or lacks channel management permission, and [`Error::Conflict`]
    /// when the channel name already exists.
    pub async fn create_channel(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        request: CreateChannelRequest,
    ) -> ruckchat_common::Result<Channel> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        self.deps
            .authorization
            .require_role_permission(membership.role, Permission::ManageChannels)?;

        let channel = Channel::new(organization_id, request.name, caller_id, request.is_private)?;
        self.deps.channels.create(&channel).await?;

        let channel_membership = ChannelMembership::new(caller_id, channel.id)?;
        self.deps.channel_memberships.create(&channel_membership).await?;

        Ok(channel)
    }

    /// Lists channels visible to the caller in an organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn list_channels_in_organization(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<Vec<Channel>> {
        let caller_membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?;
        if caller_membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        let channel_memberships = self.deps.channel_memberships.list_by_user(caller_id).await?;
        let channel_member_ids: std::collections::HashSet<ChannelId> = channel_memberships
            .into_iter()
            .map(|m| m.channel_id)
            .collect();

        let all_channels = self.deps.channels.list_by_organization(organization_id).await?;
        let visible: Vec<Channel> = all_channels
            .into_iter()
            .filter(|c| !c.is_private || channel_member_ids.contains(&c.id))
            .collect();

        Ok(visible)
    }

    /// Loads a channel if visible to the caller.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the channel does not exist or is not
    /// visible, and [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn get_channel(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
    ) -> ruckchat_common::Result<Channel> {
        let channel = self
            .deps
            .channels
            .by_id(channel_id)
            .await?
            .ok_or_else(|| Error::NotFound("channel".into()))?;

        let caller_membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let channel_membership = self
            .deps
            .channel_memberships
            .by_ids(caller_id, channel_id)
            .await?;

        self.deps
            .authorization
            .require_can_read_channel(&channel,
                caller_membership.as_ref(),
                channel_membership.as_ref(),
            )?;

        Ok(channel)
    }

    /// Updates a channel's topic and purpose.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission and
    /// [`Error::NotFound`] when the channel does not exist.
    pub async fn update_channel(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
        request: UpdateChannelRequest,
    ) -> ruckchat_common::Result<Channel> {
        let channel = self
            .deps
            .channels
            .by_id(channel_id)
            .await?
            .ok_or_else(|| Error::NotFound("channel".into()))?;

        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        self.deps
            .authorization
            .require_role_permission(membership.role, Permission::ManageChannels)?;

        let mut channel = channel;
        channel.set_topic(request.topic);
        channel.set_purpose(request.purpose);

        // Channel repository does not have an update method, so we recreate via create with ON CONFLICT.
        self.deps.channels.create(&channel).await?;
        Ok(channel)
    }

    /// Archives a channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission and
    /// [`Error::NotFound`] when the channel does not exist.
    pub async fn archive_channel(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
    ) -> ruckchat_common::Result<Channel> {
        let mut channel = self
            .deps
            .channels
            .by_id(channel_id)
            .await?
            .ok_or_else(|| Error::NotFound("channel".into()))?;

        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        self.deps
            .authorization
            .require_role_permission(membership.role, Permission::ManageChannels)?;

        channel.archive();
        self.deps.channels.create(&channel).await?;
        Ok(channel)
    }

    /// Restores an archived channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission and
    /// [`Error::NotFound`] when the channel does not exist.
    pub async fn unarchive_channel(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
    ) -> ruckchat_common::Result<Channel> {
        let mut channel = self
            .deps
            .channels
            .by_id(channel_id)
            .await?
            .ok_or_else(|| Error::NotFound("channel".into()))?;

        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        self.deps
            .authorization
            .require_role_permission(membership.role, Permission::ManageChannels)?;

        channel.unarchive();
        self.deps.channels.create(&channel).await?;
        Ok(channel)
    }

    /// Adds a user to a channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission,
    /// [`Error::NotFound`] when the channel does not exist, and [`Error::Conflict`]
    /// when the user is already a member.
    pub async fn add_member(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> ruckchat_common::Result<ChannelMembership> {
        let channel = self
            .deps
            .channels
            .by_id(channel_id)
            .await?
            .ok_or_else(|| Error::NotFound("channel".into()))?;

        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        self.deps
            .authorization
            .require_role_permission(membership.role, Permission::ManageChannels)?;

        if self
            .deps
            .channel_memberships
            .by_ids(user_id, channel_id)
            .await?
            .is_some()
        {
            return Err(Error::Conflict("user is already a channel member".into()));
        }

        let channel_membership = ChannelMembership::new(user_id, channel_id)?;
        self.deps.channel_memberships.create(&channel_membership).await?;
        Ok(channel_membership)
    }

    /// Removes a user from a channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks permission and
    /// [`Error::NotFound`] when the membership does not exist.
    pub async fn remove_member(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> ruckchat_common::Result<()> {
        let channel = self
            .deps
            .channels
            .by_id(channel_id)
            .await?
            .ok_or_else(|| Error::NotFound("channel".into()))?;

        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        self.deps
            .authorization
            .require_role_permission(membership.role, Permission::ManageChannels)?;

        self.deps
            .channel_memberships
            .delete(user_id, channel_id)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::authorization::AuthorizationService;
    use crate::services::dto::CreateChannelRequest;
    use crate::testing::{
        MockChannelMembershipRepository, MockChannelRepository,
        MockOrganizationMembershipRepository,
    };
    use ruckchat_domain::{OrganizationMembership, Role, User};
    use ruckchat_id::{OrganizationId, UserId};
    use std::sync::Arc;

    fn service() -> ChannelService {
        ChannelService::new(ChannelServiceDeps {
            channels: Arc::new(MockChannelRepository::new()),
            channel_memberships: Arc::new(MockChannelMembershipRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
            authorization: AuthorizationService::new(),
        })
    }

    async fn seed_owner_and_org(
        svc: &ChannelService,
    ) -> (UserId, OrganizationId) {
        let user = User::new("owner@example.com", "Owner", "hash").unwrap();
        let org_id = OrganizationId::new();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(user.id, org_id, Role::Owner).unwrap())
            .await
            .unwrap();
        (user.id, org_id)
    }

    #[tokio::test]
    async fn owner_can_create_channel() {
        let svc = service();
        let (owner_id, org_id) = seed_owner_and_org(&svc).await;
        let channel = svc
            .create_channel(
                owner_id,
                org_id,
                CreateChannelRequest {
                    name: "general".into(),
                    is_private: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(channel.name, "general");
    }

    #[tokio::test]
    async fn member_cannot_create_channel() {
        let svc = service();
        let (_owner_id, org_id) = seed_owner_and_org(&svc).await;
        let member = User::new("member@example.com", "Member", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(member.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .create_channel(
                member.id,
                org_id,
                CreateChannelRequest {
                    name: "random".into(),
                    is_private: false,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn private_channel_not_visible_to_non_member() {
        let svc = service();
        let (owner_id, org_id) = seed_owner_and_org(&svc).await;
        let channel = svc
            .create_channel(
                owner_id,
                org_id,
                CreateChannelRequest {
                    name: "secret".into(),
                    is_private: true,
                },
            )
            .await
            .unwrap();

        let member = User::new("member@example.com", "Member", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(member.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc.get_channel(member.id, channel.id).await.unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
