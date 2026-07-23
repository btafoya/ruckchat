//! Administrative route handlers.
//!
//! These endpoints are restricted to organization owners and admins.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse},
    migrate::MigrationData,
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_id::{FileId, OrganizationId};
use serde::Deserialize;
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

#[must_use]
fn default_true() -> bool {
    true
}
