//! Channel route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse},
    services::dto::{CreateChannelRequest, UpdateChannelRequest},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_id::{ChannelId, OrganizationId, UserId};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for adding or removing a channel member.
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelMemberParams {
    /// User to add or remove.
    pub user_id: UserId,
}

/// Lists channels visible to the caller in an organization.
pub async fn list_in_organization(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ListResponse<ruckchat_domain::Channel>>, Error> {
    let channels = state
        .channels
        .list_channels_in_organization(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(Json(ListResponse::new(channels)))
}

/// Creates a channel in an organization.
pub async fn create(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<impl IntoResponse, Error> {
    let channel = state
        .channels
        .create_channel(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(channel)))
}

/// Loads a single channel.
pub async fn get(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ruckchat_domain::Channel>, Error> {
    let channel = state
        .channels
        .get_channel(auth_user.id, ChannelId::from_uuid(channel_id))
        .await?;
    Ok(Json(channel))
}

/// Updates a channel's topic and purpose.
pub async fn update(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(request): Json<UpdateChannelRequest>,
) -> Result<Json<ruckchat_domain::Channel>, Error> {
    let channel = state
        .channels
        .update_channel(auth_user.id, ChannelId::from_uuid(channel_id), request)
        .await?;
    Ok(Json(channel))
}

/// Archives a channel.
pub async fn archive(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<ruckchat_domain::Channel>, Error> {
    let channel = state
        .channels
        .archive_channel(auth_user.id, ChannelId::from_uuid(channel_id))
        .await?;
    Ok(Json(channel))
}

/// Adds a member to a channel.
pub async fn add_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(params): Query<ChannelMemberParams>,
) -> Result<impl IntoResponse, Error> {
    let membership = state
        .channels
        .add_member(
            auth_user.id,
            ChannelId::from_uuid(channel_id),
            params.user_id,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(membership)))
}

/// Removes a member from a channel.
pub async fn remove_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(params): Query<ChannelMemberParams>,
) -> Result<StatusCode, Error> {
    state
        .channels
        .remove_member(
            auth_user.id,
            ChannelId::from_uuid(channel_id),
            params.user_id,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
