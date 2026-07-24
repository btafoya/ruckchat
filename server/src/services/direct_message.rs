//! Direct message service.

use crate::services::dto::StartDmRequest;
use ruckchat_common::Error;
use ruckchat_domain::{
    DirectMessageConversation, DirectMessageConversationRepository,
    OrganizationMembershipRepository,
};
use ruckchat_id::{DirectMessageConversationId, OrganizationId, UserId};
use std::sync::Arc;
use uuid::Uuid;

/// Dependencies required by [`DirectMessageService`].
#[derive(Clone)]
pub struct DirectMessageServiceDeps {
    /// DM conversation repository.
    pub conversations: Arc<dyn DirectMessageConversationRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn OrganizationMembershipRepository + Send + Sync>,
}

/// Direct message conversation operations.
#[derive(Clone)]
pub struct DirectMessageService {
    deps: DirectMessageServiceDeps,
}

impl DirectMessageService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: DirectMessageServiceDeps) -> Self {
        Self { deps }
    }

    /// Starts a direct message conversation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization member
    /// or includes a non-member, and [`Error::Validation`] for invalid member sets.
    pub async fn start_conversation(
        &self,
        caller_id: UserId,
        request: StartDmRequest,
    ) -> ruckchat_common::Result<DirectMessageConversation> {
        let caller_membership = self
            .deps
            .memberships
            .by_ids(caller_id, request.organization_id)
            .await?;
        if caller_membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        let mut member_ids = request.member_ids;
        member_ids.push(caller_id);
        member_ids.sort_unstable();
        member_ids.dedup();

        for user_id in &member_ids {
            if user_id == &caller_id {
                continue;
            }
            let membership = self
                .deps
                .memberships
                .by_ids(*user_id, request.organization_id)
                .await?;
            if membership.is_none() {
                return Err(Error::Forbidden(
                    "all participants must be organization members".into(),
                ));
            }
        }

        if let Some(existing) = self
            .deps
            .conversations
            .find_by_members(request.organization_id, &member_ids)
            .await?
        {
            self.deps
                .conversations
                .unhide(caller_id, existing.id)
                .await?;
            return Ok(existing);
        }

        let conversation = DirectMessageConversation::new(request.organization_id, member_ids)?;
        self.deps.conversations.create(&conversation).await?;
        Ok(conversation)
    }

    /// Hides a conversation from the caller's own conversation list. The
    /// conversation remains visible to other members and reappears for the
    /// caller once a new message is posted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the conversation does not exist and
    /// [`Error::Forbidden`] when the caller is not a member.
    pub async fn hide_conversation(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
    ) -> ruckchat_common::Result<()> {
        let id = DirectMessageConversationId::from_uuid(conversation_id);
        let conversation = self
            .deps
            .conversations
            .by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound("conversation".into()))?;

        if !conversation.member_ids.contains(&caller_id) {
            return Err(Error::Forbidden(
                "must be a conversation member to hide it".into(),
            ));
        }

        self.deps.conversations.hide(caller_id, id).await
    }

    /// Lists DM conversations for the caller in an organization.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization member.
    pub async fn list_conversations_for_user(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<Vec<DirectMessageConversation>> {
        let caller_membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?;
        if caller_membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        self.deps
            .conversations
            .list_by_user_and_organization(caller_id, organization_id)
            .await
    }

    /// Loads a DM conversation the caller participates in.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the conversation does not exist or the
    /// caller is not a member, and [`Error::Forbidden`] when the caller is not an
    /// organization member.
    pub async fn get_conversation(
        &self,
        caller_id: UserId,
        conversation_id: Uuid,
    ) -> ruckchat_common::Result<DirectMessageConversation> {
        let conversation_id = DirectMessageConversationId::from_uuid(conversation_id);
        let conversation = self
            .deps
            .conversations
            .by_id(conversation_id)
            .await?
            .ok_or_else(|| Error::NotFound("conversation".into()))?;

        let caller_membership = self
            .deps
            .memberships
            .by_ids(caller_id, conversation.organization_id)
            .await?;
        if caller_membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }
        if !conversation.member_ids.contains(&caller_id) {
            return Err(Error::Forbidden(
                "must be a conversation member to read".into(),
            ));
        }
        Ok(conversation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dto::StartDmRequest;
    use crate::testing::{
        MockDirectMessageConversationRepository, MockOrganizationMembershipRepository,
    };
    use ruckchat_domain::{OrganizationMembership, Role, User};

    fn service() -> DirectMessageService {
        DirectMessageService::new(DirectMessageServiceDeps {
            conversations: Arc::new(MockDirectMessageConversationRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
        })
    }

    async fn seed_members(
        svc: &DirectMessageService,
        org_id: OrganizationId,
        count: usize,
    ) -> Vec<UserId> {
        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let user =
                User::new(format!("dm{i}@example.com"), format!("User {i}"), "hash").unwrap();
            svc.deps
                .memberships
                .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
                .await
                .unwrap();
            ids.push(user.id);
        }
        ids
    }

    #[tokio::test]
    async fn starting_dm_twice_reuses_conversation() {
        let svc = service();
        let org_id = OrganizationId::new();
        let members = seed_members(&svc, org_id, 2).await;
        let (caller, other) = (members[0], members[1]);

        let first = svc
            .start_conversation(
                caller,
                StartDmRequest {
                    organization_id: org_id,
                    member_ids: vec![other],
                },
            )
            .await
            .unwrap();
        let second = svc
            .start_conversation(
                caller,
                StartDmRequest {
                    organization_id: org_id,
                    member_ids: vec![other],
                },
            )
            .await
            .unwrap();

        assert_eq!(first.id, second.id);
    }

    #[tokio::test]
    async fn hidden_conversation_excluded_then_reappears_on_restart() {
        let svc = service();
        let org_id = OrganizationId::new();
        let members = seed_members(&svc, org_id, 2).await;
        let (caller, other) = (members[0], members[1]);

        let conversation = svc
            .start_conversation(
                caller,
                StartDmRequest {
                    organization_id: org_id,
                    member_ids: vec![other],
                },
            )
            .await
            .unwrap();

        svc.hide_conversation(caller, conversation.id.as_uuid())
            .await
            .unwrap();
        let visible = svc
            .list_conversations_for_user(caller, org_id)
            .await
            .unwrap();
        assert!(visible.is_empty());

        svc.start_conversation(
            caller,
            StartDmRequest {
                organization_id: org_id,
                member_ids: vec![other],
            },
        )
        .await
        .unwrap();
        let visible = svc
            .list_conversations_for_user(caller, org_id)
            .await
            .unwrap();
        assert_eq!(visible.len(), 1);
    }

    #[tokio::test]
    async fn non_member_cannot_hide_conversation() {
        let svc = service();
        let org_id = OrganizationId::new();
        let members = seed_members(&svc, org_id, 2).await;
        let (caller, other) = (members[0], members[1]);

        let conversation = svc
            .start_conversation(
                caller,
                StartDmRequest {
                    organization_id: org_id,
                    member_ids: vec![other],
                },
            )
            .await
            .unwrap();

        let outsider = UserId::new();
        let err = svc
            .hide_conversation(outsider, conversation.id.as_uuid())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
