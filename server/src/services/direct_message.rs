//! Direct message service.

use crate::services::dto::StartDmRequest;
use ruckchat_common::Error;
use ruckchat_domain::{
    DirectMessageConversation, DirectMessageConversationRepository, OrganizationMembershipRepository,
};
use ruckchat_id::{OrganizationId, UserId};
use std::sync::Arc;

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

        let conversation =
            DirectMessageConversation::new(request.organization_id, member_ids)?;
        self.deps.conversations.create(&conversation).await?;
        Ok(conversation)
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
}
