//! Authorization service.
//!
//! Encapsulates permission checks used by other services. Permissions are
//! derived from the caller's organization role and resource ownership.

use ruckchat_common::Error;
use ruckchat_domain::{Channel, Message, OrganizationMembership, Role};
use ruckchat_id::UserId;

/// Fine-grained permissions within an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Manage organization-wide settings and members.
    ManageOrganization,
    /// Create, archive, or configure channels.
    ManageChannels,
    /// Delete or edit any message regardless of authorship.
    ModerateMessages,
    /// Post in a channel.
    PostInChannel,
    /// Read messages in a channel.
    ReadChannel,
    /// Start or participate in direct messages.
    DirectMessage,
}

/// Decides whether an action is allowed.
#[derive(Debug, Clone, Default)]
pub struct AuthorizationService;

impl AuthorizationService {
    /// Creates the authorization service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Checks whether the caller holds the requested permission given their
    /// organization role. This variant ignores resource ownership.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller lacks the permission.
    pub fn require_role_permission(
        &self,
        role: Role,
        permission: Permission,
    ) -> ruckchat_common::Result<()> {
        if self.has_role_permission(role, permission) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "missing permission: {permission:?}"
            )))
        }
    }

    fn has_role_permission(&self, role: Role, permission: Permission) -> bool {
        match permission {
            Permission::ManageOrganization | Permission::ManageChannels => role.is_manager(),
            Permission::ModerateMessages => role.is_moderator(),
            Permission::PostInChannel | Permission::ReadChannel | Permission::DirectMessage => {
                // Any organization member can read public channels and DM.
                matches!(role, Role::Owner | Role::Admin | Role::Member)
            }
        }
    }

    /// Checks whether the caller can edit a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is neither the author nor a
    /// moderator.
    pub fn require_can_edit_message(
        &self,
        message: &Message,
        caller_id: UserId,
        caller_role: Role,
    ) -> ruckchat_common::Result<()> {
        if message.author_id == caller_id || caller_role.is_moderator() {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "only the author or a moderator can edit messages".into(),
            ))
        }
    }

    /// Checks whether the caller can delete a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is neither the author nor a
    /// moderator.
    pub fn require_can_delete_message(
        &self,
        message: &Message,
        caller_id: UserId,
        caller_role: Role,
    ) -> ruckchat_common::Result<()> {
        if message.author_id == caller_id || caller_role.is_moderator() {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "only the author or a moderator can delete messages".into(),
            ))
        }
    }

    /// Returns true when the user is a server administrator.
    #[must_use]
    pub fn is_server_admin(&self, user: &ruckchat_domain::User) -> bool {
        user.is_server_admin
    }

    /// Requires the user to be a server administrator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the user is not a server admin.
    pub fn require_server_admin(
        &self,
        user: &ruckchat_domain::User,
    ) -> ruckchat_common::Result<()> {
        if user.is_server_admin {
            Ok(())
        } else {
            Err(Error::Forbidden("server admin access required".into()))
        }
    }

    /// Checks whether the caller can post in a channel.
    ///
    /// Posting requires the caller to be both an organization member and an
    /// explicit member of the channel. Reading public channels is less strict
    /// (see [`Self::require_can_read_channel`]).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when posting is not allowed.
    pub fn require_can_post_in_channel(
        &self,
        _channel: &Channel,
        caller_membership: Option<&OrganizationMembership>,
        channel_membership: Option<&ruckchat_domain::ChannelMembership>,
    ) -> ruckchat_common::Result<()> {
        caller_membership
            .ok_or_else(|| Error::Forbidden("must be an organization member to post".into()))?;

        if channel_membership.is_none() {
            return Err(Error::Forbidden(
                "must be a member of the channel to post".into(),
            ));
        }

        Ok(())
    }

    /// Checks whether the caller can read a channel.
    ///
    /// Public channels are readable by any organization member. Private channels
    /// require explicit membership.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when reading is not allowed.
    pub fn require_can_read_channel(
        &self,
        channel: &Channel,
        caller_membership: Option<&OrganizationMembership>,
        channel_membership: Option<&ruckchat_domain::ChannelMembership>,
    ) -> ruckchat_common::Result<()> {
        caller_membership.ok_or_else(|| {
            Error::Forbidden("must be an organization member to read channels".into())
        })?;

        if channel.is_private && channel_membership.is_none() {
            return Err(Error::Forbidden(
                "must be a member of the private channel to read".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruckchat_domain::{Channel, Message, OrganizationMembership, Role};
    use ruckchat_id::{OrganizationId, UserId};
    use uuid::Uuid;

    fn authz() -> AuthorizationService {
        AuthorizationService::new()
    }

    fn membership(role: Role) -> OrganizationMembership {
        OrganizationMembership::new(UserId::new(), OrganizationId::new(), role).unwrap()
    }

    fn channel(is_private: bool) -> Channel {
        Channel::new(OrganizationId::new(), "general", UserId::new(), is_private).unwrap()
    }

    fn message(author_id: UserId) -> Message {
        Message::new(
            Uuid::new_v4(),
            ruckchat_domain::ConversationType::Channel,
            author_id,
            "hello",
            None,
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn owner_can_manage_organization() {
        assert!(
            authz()
                .require_role_permission(Role::Owner, Permission::ManageOrganization)
                .is_ok()
        );
    }

    #[test]
    fn member_cannot_manage_organization() {
        assert!(
            authz()
                .require_role_permission(Role::Member, Permission::ManageOrganization)
                .is_err()
        );
    }

    #[test]
    fn author_can_edit_own_message() {
        let author = UserId::new();
        let msg = message(author);
        assert!(
            authz()
                .require_can_edit_message(&msg, author, Role::Member)
                .is_ok()
        );
    }

    #[test]
    fn non_author_cannot_edit_message() {
        let author = UserId::new();
        let other = UserId::new();
        let msg = message(author);
        assert!(
            authz()
                .require_can_edit_message(&msg, other, Role::Member)
                .is_err()
        );
    }

    #[test]
    fn moderator_can_edit_any_message() {
        let author = UserId::new();
        let other = UserId::new();
        let msg = message(author);
        assert!(
            authz()
                .require_can_edit_message(&msg, other, Role::Admin)
                .is_ok()
        );
    }

    #[test]
    fn public_channel_readable_by_member() {
        let ch = channel(false);
        let membership = membership(Role::Member);
        assert!(
            authz()
                .require_can_read_channel(&ch, Some(&membership), None)
                .is_ok()
        );
    }

    #[test]
    fn private_channel_not_readable_without_membership() {
        let ch = channel(true);
        let membership = membership(Role::Member);
        assert!(
            authz()
                .require_can_read_channel(&ch, Some(&membership), None)
                .is_err()
        );
    }
}
