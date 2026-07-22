//! Organization route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse},
    services::dto::{ChangeRoleRequest, CreateOrganizationRequest, InviteMemberRequest},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_id::OrganizationId;
use serde::Deserialize;
use uuid::Uuid;

/// Lists organizations the authenticated user belongs to.
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<ListResponse<ruckchat_domain::Organization>>, Error> {
    let organizations = state.organizations.list_for_user(auth_user.id).await?;
    Ok(Json(ListResponse::new(organizations)))
}

/// Creates a new organization owned by the authenticated user.
pub async fn create(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, Error> {
    let organization = state
        .organizations
        .create_organization(auth_user.id, request)
        .await?;
    Ok((StatusCode::CREATED, Json(organization)))
}

/// Invites an existing user to the organization.
pub async fn invite_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<InviteMemberRequest>,
) -> Result<impl IntoResponse, Error> {
    let membership = state
        .organizations
        .invite_member(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(membership)))
}

/// Changes a member's role in the organization.
pub async fn change_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<ChangeRoleRequest>,
) -> Result<StatusCode, Error> {
    state
        .organizations
        .change_member_role(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request,
        )
        .await?;
    Ok(StatusCode::OK)
}

/// Query parameters for removing a member from an organization.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoveMemberParams {
    /// User to remove.
    pub user_id: ruckchat_id::UserId,
}

/// Removes a member from the organization.
pub async fn remove_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Query(params): Query<RemoveMemberParams>,
) -> Result<StatusCode, Error> {
    state
        .organizations
        .remove_member(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            params.user_id,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
