//! Administrative route handlers.
//!
//! These endpoints are restricted to organization owners and admins.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse, dto::UserResponse},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_id::{
    ChannelId, CustomEmojiId, FileId, OrganizationId, OrganizationRoleId, PermissionId, TeamId,
    UserId,
};
use ruckchat_migrate::MigrationData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Imports a migration snapshot into the target organization.
pub async fn import(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<AdminImportRequest>,
) -> Result<impl IntoResponse, Error> {
    let counts = state
        .admin
        .import_snapshot(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            &request.data,
            request.dry_run,
        )
        .await?;

    Ok((
        StatusCode::OK,
        Json(ImportCountsResponse {
            inserted: counts.inserted,
            skipped: counts.skipped,
        }),
    ))
}

/// Lists custom roles defined in the organization.
pub async fn list_roles(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ListResponse<ruckchat_domain::OrganizationRole>>, Error> {
    let roles = state
        .admin
        .list_roles(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(Json(ListResponse::new(roles)))
}

/// Creates a custom role in the organization.
pub async fn create_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, Error> {
    let role = state
        .admin
        .create_role(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request.name,
            request.description,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(role)))
}

/// Lists permissions defined in the organization.
pub async fn list_permissions(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ListResponse<ruckchat_domain::Permission>>, Error> {
    let permissions = state
        .admin
        .list_permissions(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(Json(ListResponse::new(permissions)))
}

/// Creates a permission in the organization.
pub async fn create_permission(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<CreatePermissionRequest>,
) -> Result<impl IntoResponse, Error> {
    let permission = state
        .admin
        .create_permission(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request.key,
            request.description,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(permission)))
}

/// Lists custom emoji defined in the organization.
pub async fn list_emoji(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ListResponse<ruckchat_domain::CustomEmoji>>, Error> {
    let emoji = state
        .admin
        .list_emoji(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(Json(ListResponse::new(emoji)))
}

/// Creates a custom emoji in the organization.
pub async fn create_emoji(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<CreateEmojiRequest>,
) -> Result<impl IntoResponse, Error> {
    let emoji = state
        .admin
        .create_emoji(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request.shortcode,
            FileId::from_uuid(request.file_id),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(emoji)))
}

/// Lists teams defined in the organization.
pub async fn list_teams(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ListResponse<ruckchat_domain::Team>>, Error> {
    let teams = state
        .admin
        .list_teams(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(Json(ListResponse::new(teams)))
}

/// Creates a team in the organization.
pub async fn create_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<CreateTeamRequest>,
) -> Result<impl IntoResponse, Error> {
    let team = state
        .admin
        .create_team(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request.name,
            request.description,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(team)))
}

/// Request to import a migration snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminImportRequest {
    /// Migration snapshot to import.
    pub data: MigrationData,
    /// When true, validate the snapshot without writing to the database.
    #[serde(default = "default_true")]
    pub dry_run: bool,
}

/// Result of an import operation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportCountsResponse {
    /// Rows inserted or updated.
    pub inserted: usize,
    /// Rows skipped because they already existed.
    pub skipped: usize,
}

/// Request to create a custom role.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRoleRequest {
    /// Role name unique within the organization.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Request to create a permission.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePermissionRequest {
    /// Machine-readable permission key.
    pub key: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Request to create a custom emoji.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEmojiRequest {
    /// Shortcode without surrounding colons.
    pub shortcode: String,
    /// Identifier of the uploaded image file.
    pub file_id: Uuid,
}

/// Request to create a team.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTeamRequest {
    /// Team name unique within the organization.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Loads organization settings.
pub async fn get_organization_settings(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<Json<ruckchat_domain::OrganizationSettings>, Error> {
    let settings = state
        .admin
        .get_organization_settings(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(Json(settings))
}

/// Updates organization settings.
pub async fn update_organization_settings(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<UpdateOrganizationSettingsRequest>,
) -> Result<Json<ruckchat_domain::OrganizationSettings>, Error> {
    let settings = state
        .admin
        .update_organization_settings(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request.max_file_size_bytes,
            request.storage_quota_bytes,
        )
        .await?;
    Ok(Json(settings))
}

/// Updates a custom role.
pub async fn update_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, role_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<ruckchat_domain::OrganizationRole>, Error> {
    let role = state
        .admin
        .update_role(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            OrganizationRoleId::from_uuid(role_id),
            request.name,
            request.description,
        )
        .await?;
    Ok(Json(role))
}

/// Deletes a custom role.
pub async fn delete_role(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    state
        .admin
        .delete_role(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            OrganizationRoleId::from_uuid(role_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Updates a permission.
pub async fn update_permission(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, permission_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdatePermissionRequest>,
) -> Result<Json<ruckchat_domain::Permission>, Error> {
    let permission = state
        .admin
        .update_permission(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            PermissionId::from_uuid(permission_id),
            request.key,
            request.description,
        )
        .await?;
    Ok(Json(permission))
}

/// Deletes a permission.
pub async fn delete_permission(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, permission_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    state
        .admin
        .delete_permission(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            PermissionId::from_uuid(permission_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a custom emoji.
pub async fn delete_emoji(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, emoji_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    state
        .admin
        .delete_emoji(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            CustomEmojiId::from_uuid(emoji_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Updates a team.
pub async fn update_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateTeamRequest>,
) -> Result<Json<ruckchat_domain::Team>, Error> {
    let team = state
        .admin
        .update_team(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
            request.name,
            request.description,
        )
        .await?;
    Ok(Json(team))
}

/// Deletes a team.
pub async fn delete_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    state
        .admin
        .delete_team(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Lists members of a team.
pub async fn list_team_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ListResponse<TeamMemberResponse>>, Error> {
    let members = state
        .admin
        .list_team_members(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
        )
        .await?;
    let items = members
        .into_iter()
        .map(|(membership, user)| TeamMemberResponse {
            user: UserResponse::from_domain(&user),
            role: membership.role,
        })
        .collect();
    Ok(Json(ListResponse::new(items)))
}

/// Adds a user to a team.
pub async fn add_team_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AddTeamMemberRequest>,
) -> Result<impl IntoResponse, Error> {
    let membership = state
        .admin
        .add_team_member(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
            UserId::from_uuid(request.user_id),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(membership)))
}

/// Removes a user from a team.
pub async fn remove_team_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id, user_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    state
        .admin
        .remove_team_member(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
            UserId::from_uuid(user_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Lists channels linked to a team.
pub async fn list_team_rooms(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ListResponse<ruckchat_domain::TeamRoom>>, Error> {
    let rooms = state
        .admin
        .list_team_rooms(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
        )
        .await?;
    Ok(Json(ListResponse::new(rooms)))
}

/// Links a channel to a team.
pub async fn add_team_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AddTeamRoomRequest>,
) -> Result<impl IntoResponse, Error> {
    let link = state
        .admin
        .add_team_room(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
            ChannelId::from_uuid(request.channel_id),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(link)))
}

/// Unlinks a channel from a team.
pub async fn remove_team_room(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((organization_id, team_id, channel_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    state
        .admin
        .remove_team_room(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            TeamId::from_uuid(team_id),
            ChannelId::from_uuid(channel_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Team member response.
#[derive(Debug, Clone, Serialize)]
pub struct TeamMemberResponse {
    /// Public user information.
    pub user: UserResponse,
    /// Role within the team.
    pub role: ruckchat_domain::TeamRole,
}

/// Request to add a user to a team.
#[derive(Debug, Clone, Deserialize)]
pub struct AddTeamMemberRequest {
    /// User to add.
    pub user_id: Uuid,
}

/// Request to link a channel to a team.
#[derive(Debug, Clone, Deserialize)]
pub struct AddTeamRoomRequest {
    /// Channel to link.
    pub channel_id: Uuid,
}

/// Request to update organization settings.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateOrganizationSettingsRequest {
    /// Maximum file upload size in bytes.
    pub max_file_size_bytes: i64,
    /// Total storage quota in bytes.
    pub storage_quota_bytes: i64,
}

/// Request to update a custom role.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRoleRequest {
    /// Role name unique within the organization.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Request to update a permission.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePermissionRequest {
    /// Machine-readable permission key.
    pub key: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Request to update a team.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamRequest {
    /// Team name unique within the organization.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

#[must_use]
fn default_true() -> bool {
    true
}
