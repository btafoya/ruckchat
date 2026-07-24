//! Organization route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse, dto::UserResponse},
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

/// Query parameters for searching organization members.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct SearchMembersQuery {
    /// Optional filter by display name or email prefix.
    pub q: String,
}

/// Lists members of the organization.
pub async fn list_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ListResponse<MemberResponse>>, Error> {
    let members = state
        .organizations
        .list_members(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    let items = members
        .into_iter()
        .map(|(membership, user)| MemberResponse {
            user: UserResponse::from_domain(&user),
            role: membership.role,
        })
        .collect();
    Ok(Json(ListResponse::new(items)))
}

/// Searches organization members by display name or email.
pub async fn search_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Query(query): Query<SearchMembersQuery>,
) -> Result<Json<ListResponse<UserResponse>>, Error> {
    let users = state
        .organizations
        .search_members(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            &query.q,
        )
        .await?;
    let items = users
        .into_iter()
        .map(|user| UserResponse::from_domain(&user))
        .collect();
    Ok(Json(ListResponse::new(items)))
}

/// Organization member response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemberResponse {
    /// Public user information.
    pub user: UserResponse,
    /// Role within the organization.
    pub role: ruckchat_domain::Role,
}
