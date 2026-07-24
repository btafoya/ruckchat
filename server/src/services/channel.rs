//! Channel service.

use crate::services::authorization::AuthorizationService;
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
    /// member, and [`Error::Conflict`] when the channel name already exists.
    pub async fn create_channel(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        request: CreateChannelRequest,
    ) -> ruckchat_common::Result<Channel> {
        if self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?
            .is_none()
        {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        let channel = Channel::new(organization_id, request.name, caller_id, request.is_private)?;
        self.deps.channels.create(&channel).await?;

        let channel_membership = ChannelMembership::new(caller_id, channel.id)?;
        self.deps
            .channel_memberships
            .create(&channel_membership)
            .await?;

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

        let channel_memberships = self
            .deps
            .channel_memberships
            .list_by_user(caller_id)
            .await?;
        let channel_member_ids: std::collections::HashSet<ChannelId> = channel_memberships
            .into_iter()
            .map(|m| m.channel_id)
            .collect();

        let all_channels = self
            .deps
            .channels
            .list_by_organization(organization_id)
            .await?;
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

        self.deps.authorization.require_can_read_channel(
            &channel,
            caller_membership.as_ref(),
            channel_membership.as_ref(),
        )?;

        Ok(channel)
    }

    /// Lists the explicit members of a channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the channel does not exist, and
    /// [`Error::Forbidden`] when the caller cannot read the channel.
    pub async fn list_members(
        &self,
        caller_id: UserId,
        channel_id: ChannelId,
    ) -> ruckchat_common::Result<Vec<ChannelMembership>> {
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

        self.deps.authorization.require_can_read_channel(
            &channel,
            caller_membership.as_ref(),
            channel_membership.as_ref(),
        )?;

        self.deps
            .channel_memberships
            .list_by_channel(channel_id)
            .await
    }

    /// Requires the caller to be either the channel's creator or an
    /// organization manager (owner/admin).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization
    /// member, or is neither the creator nor a manager.
    async fn require_can_manage_channel(
        &self,
        caller_id: UserId,
        channel: &Channel,
    ) -> ruckchat_common::Result<()> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?;
        let Some(membership) = membership else {
            return Err(Error::Forbidden("must be an organization member".into()));
        };

        if channel.created_by == caller_id || membership.role.is_manager() {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "must be the channel creator or an organization manager".into(),
            ))
        }
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

        self.require_can_manage_channel(caller_id, &channel).await?;

        let mut channel = channel;
        channel.set_topic(request.topic);
        channel.set_purpose(request.purpose);

        self.deps.channels.update(&channel).await?;
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

        self.require_can_manage_channel(caller_id, &channel).await?;

        channel.archive();
        self.deps.channels.update(&channel).await?;
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

        self.require_can_manage_channel(caller_id, &channel).await?;

        channel.unarchive();
        self.deps.channels.update(&channel).await?;
        Ok(channel)
    }

    /// Adds a user to a channel.
    ///
    /// Any organization member may join a public channel themselves. Inviting
    /// another user, or adding anyone to a private channel, requires being
    /// the channel's creator or an organization manager.
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

        let self_join = caller_id == user_id && !channel.is_private;
        if !self_join {
            self.require_can_manage_channel(caller_id, &channel).await?;
        } else if self
            .deps
            .memberships
            .by_ids(caller_id, channel.organization_id)
            .await?
            .is_none()
        {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

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
        self.deps
            .channel_memberships
            .create(&channel_membership)
            .await?;
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

        self.require_can_manage_channel(caller_id, &channel).await?;

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

    async fn seed_owner_and_org(svc: &ChannelService) -> (UserId, OrganizationId) {
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
    async fn member_can_create_channel() {
        let svc = service();
        let (_owner_id, org_id) = seed_owner_and_org(&svc).await;
        let member = User::new("member@example.com", "Member", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(member.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let channel = svc
            .create_channel(
                member.id,
                org_id,
                CreateChannelRequest {
                    name: "random".into(),
                    is_private: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(channel.created_by, member.id);
    }

    #[tokio::test]
    async fn outsider_cannot_create_channel() {
        let svc = service();
        let (_owner_id, org_id) = seed_owner_and_org(&svc).await;
        let outsider = UserId::new();

        let err = svc
            .create_channel(
                outsider,
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
    async fn channel_creator_can_update_own_channel() {
        let svc = service();
        let (_owner_id, org_id) = seed_owner_and_org(&svc).await;
        let member = User::new("member@example.com", "Member", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(member.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let channel = svc
            .create_channel(
                member.id,
                org_id,
                CreateChannelRequest {
                    name: "random".into(),
                    is_private: false,
                },
            )
            .await
            .unwrap();

        let updated = svc
            .update_channel(
                member.id,
                channel.id,
                UpdateChannelRequest {
                    topic: Some("New topic".into()),
                    purpose: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.topic.as_deref(), Some("New topic"));
    }

    #[tokio::test]
    async fn non_creator_member_cannot_update_channel() {
        let svc = service();
        let (owner_id, org_id) = seed_owner_and_org(&svc).await;
        let channel = svc
            .create_channel(
                owner_id,
                org_id,
                CreateChannelRequest {
                    name: "random".into(),
                    is_private: false,
                },
            )
            .await
            .unwrap();
        let other_member = User::new("other@example.com", "Other", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(other_member.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .update_channel(
                other_member.id,
                channel.id,
                UpdateChannelRequest {
                    topic: Some("New topic".into()),
                    purpose: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn member_can_self_join_public_channel() {
        let svc = service();
        let (owner_id, org_id) = seed_owner_and_org(&svc).await;
        let channel = svc
            .create_channel(
                owner_id,
                org_id,
                CreateChannelRequest {
                    name: "random".into(),
                    is_private: false,
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

        svc.add_member(member.id, channel.id, member.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn member_cannot_self_join_private_channel() {
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

        let err = svc
            .add_member(member.id, channel.id, member.id)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn member_cannot_invite_others_to_channel_they_did_not_create() {
        let svc = service();
        let (owner_id, org_id) = seed_owner_and_org(&svc).await;
        let channel = svc
            .create_channel(
                owner_id,
                org_id,
                CreateChannelRequest {
                    name: "random".into(),
                    is_private: false,
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
        let other = User::new("other@example.com", "Other", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(other.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let err = svc
            .add_member(member.id, channel.id, other.id)
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
