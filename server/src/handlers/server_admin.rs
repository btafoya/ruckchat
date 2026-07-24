//! Server-wide administrative route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse},
    services::dto::{CreateOrganizationRequest, Pagination},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_domain::ServerSettings;
use ruckchat_id::{OrganizationId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lists all organizations on the server.
pub async fn list_organizations(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<ListResponse<ruckchat_domain::Organization>>, Error> {
    let organizations = state.server_admin.list_organizations(auth_user.id).await?;
    Ok(Json(ListResponse::new(organizations)))
}

/// Creates a new organization without joining it.
pub async fn create_organization(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, Error> {
    let organization = state
        .server_admin
        .create_organization(auth_user.id, request)
        .await?;
    Ok((StatusCode::CREATED, Json(organization)))
}

/// Renames an organization.
pub async fn rename_organization(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Json(request): Json<RenameOrganizationRequest>,
) -> Result<Json<ruckchat_domain::Organization>, Error> {
    let organization = state
        .server_admin
        .rename_organization(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            request.name,
        )
        .await?;
    Ok(Json(organization))
}

/// Deletes an organization.
pub async fn delete_organization(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    state
        .server_admin
        .delete_organization(auth_user.id, OrganizationId::from_uuid(organization_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Query parameters for listing users.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ListUsersQuery {
    /// Maximum number of users to return.
    pub limit: i64,
    /// Number of users to skip.
    pub offset: i64,
    /// When true, only return server administrators.
    pub is_server_admin: Option<bool>,
}

impl Default for ListUsersQuery {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
            is_server_admin: None,
        }
    }
}

/// Creates a new user account without an organization.
pub async fn create_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<CreateServerUserRequest>,
) -> Result<impl IntoResponse, Error> {
    let (user, password) = state
        .server_admin
        .create_user(
            auth_user.id,
            request.email,
            request.display_name,
            request.password,
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateServerUserResponse {
            user: ServerUserResponse::from_domain(&user),
            password,
        }),
    ))
}

/// Lists all users on the server.
pub async fn list_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ListResponse<ServerUserResponse>>, Error> {
    let pagination = Pagination {
        limit: query.limit,
        offset: query.offset,
    };
    let users = state
        .server_admin
        .list_users(auth_user.id, pagination)
        .await?;
    let items = users
        .into_iter()
        .filter(|u| {
            query
                .is_server_admin
                .is_none_or(|expected| u.is_server_admin == expected)
        })
        .map(|user| ServerUserResponse::from_domain(&user))
        .collect();
    Ok(Json(ListResponse::new(items)))
}

/// Loads a user by id.
pub async fn get_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ServerUserResponse>, Error> {
    let user = state
        .server_admin
        .get_user(auth_user.id, UserId::from_uuid(user_id))
        .await?;
    Ok(Json(ServerUserResponse::from_domain(&user)))
}

/// Updates a user's profile.
pub async fn update_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateServerUserRequest>,
) -> Result<Json<ServerUserResponse>, Error> {
    let user = state
        .server_admin
        .update_user(
            auth_user.id,
            UserId::from_uuid(user_id),
            request.display_name,
            request.avatar_url,
            request.email,
        )
        .await?;
    Ok(Json(ServerUserResponse::from_domain(&user)))
}

/// Resets a user's password to a server-generated value.
pub async fn reset_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ResetPasswordResponse>, Error> {
    let password = state
        .server_admin
        .reset_password(auth_user.id, UserId::from_uuid(user_id))
        .await?;
    Ok(Json(ResetPasswordResponse { password }))
}

/// Promotes a user to server administrator.
pub async fn promote_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ServerUserResponse>, Error> {
    let user = state
        .server_admin
        .promote_user(auth_user.id, UserId::from_uuid(user_id))
        .await?;
    Ok(Json(ServerUserResponse::from_domain(&user)))
}

/// Demotes a user from server administrator.
pub async fn demote_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ServerUserResponse>, Error> {
    let user = state
        .server_admin
        .demote_user(auth_user.id, UserId::from_uuid(user_id))
        .await?;
    Ok(Json(ServerUserResponse::from_domain(&user)))
}

/// Deactivates a user account.
pub async fn deactivate_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ServerUserResponse>, Error> {
    let user = state
        .server_admin
        .deactivate_user(auth_user.id, UserId::from_uuid(user_id))
        .await?;
    Ok(Json(ServerUserResponse::from_domain(&user)))
}

/// Reactivates a previously deactivated user account.
pub async fn reactivate_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ServerUserResponse>, Error> {
    let user = state
        .server_admin
        .reactivate_user(auth_user.id, UserId::from_uuid(user_id))
        .await?;
    Ok(Json(ServerUserResponse::from_domain(&user)))
}

/// Lists current server administrators.
pub async fn list_server_admins(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<ListResponse<ServerUserResponse>>, Error> {
    let users = state.server_admin.list_server_admins(auth_user.id).await?;
    let items = users
        .into_iter()
        .map(|user| ServerUserResponse::from_domain(&user))
        .collect();
    Ok(Json(ListResponse::new(items)))
}

/// Returns the merged server settings.
pub async fn get_settings(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<ServerSettings>, Error> {
    let settings = state.server_settings.load().await?;
    // Reading settings is restricted to server admins even though the merged
    // values are global.
    state
        .server_admin
        .require_server_admin(auth_user.id)
        .await?;
    Ok(Json(settings))
}

/// Updates server settings in the database.
pub async fn update_settings(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<UpdateServerSettingsRequest>,
) -> Result<StatusCode, Error> {
    let settings = ServerSettings {
        maintenance_mode_enabled: request.maintenance_mode_enabled,
        default_max_file_size_bytes: request.default_max_file_size_bytes,
        default_storage_quota_bytes: request.default_storage_quota_bytes,
        allowed_signup_domains: request.allowed_signup_domains,
        allow_registration: request.allow_registration,
        spelling_enabled: request.spelling_enabled,
        spelling_default_language: request.spelling_default_language,
    };
    state.server_settings.save(&settings, auth_user.id).await?;
    Ok(StatusCode::OK)
}

/// Query parameters for the audit log.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuditLogQuery {
    /// Filter by actor user id.
    pub actor_id: Option<Uuid>,
    /// Filter by target organization id.
    pub organization_id: Option<Uuid>,
    /// Filter by action type.
    pub action: Option<String>,
    /// Filter by resource type.
    pub resource_type: Option<String>,
    /// Start of the time range (inclusive).
    #[serde(with = "time::serde::rfc3339::option")]
    pub from: Option<time::OffsetDateTime>,
    /// End of the time range (inclusive).
    #[serde(with = "time::serde::rfc3339::option")]
    pub to: Option<time::OffsetDateTime>,
    /// Maximum entries to return.
    pub limit: i64,
    /// Number of entries to skip.
    pub offset: i64,
}

impl Default for AuditLogQuery {
    fn default() -> Self {
        Self {
            actor_id: None,
            organization_id: None,
            action: None,
            resource_type: None,
            from: None,
            to: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// Queries the global audit log.
pub async fn get_audit_log(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<ListResponse<ruckchat_domain::AuditLogEntry>>, Error> {
    // Audit-log access is restricted to server admins.
    state
        .server_admin
        .require_server_admin(auth_user.id)
        .await?;
    let entries = state
        .audit
        .query(
            query.actor_id.map(UserId::from_uuid),
            query.organization_id.map(OrganizationId::from_uuid),
            query.action.as_deref(),
            query.resource_type.as_deref(),
            query.from,
            query.to,
            query.limit.clamp(1, 100),
            query.offset.max(0),
        )
        .await?;
    Ok(Json(ListResponse::new(entries)))
}

/// Starts an impersonation session for a target user.
pub async fn impersonate(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<ImpersonateRequest>,
) -> Result<Json<ImpersonateResponse>, Error> {
    let token = state
        .server_admin
        .impersonate(auth_user.id, UserId::from_uuid(request.target_user_id))
        .await?;
    Ok(Json(ImpersonateResponse { token }))
}

/// Ends an impersonation session by token.
pub async fn end_impersonate(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<EndImpersonateRequest>,
) -> Result<StatusCode, Error> {
    state
        .server_admin
        .end_impersonate(auth_user.id, &request.token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Server-admin view of a user.
#[derive(Debug, Clone, Serialize)]
pub struct ServerUserResponse {
    /// Internal user identifier.
    pub id: UserId,
    /// Globally unique email address.
    pub email: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Optional URL to an avatar image.
    pub avatar_url: Option<String>,
    /// Whether the user is a server-wide administrator.
    pub is_server_admin: bool,
    /// Timestamp when the user was deactivated, if applicable.
    #[serde(with = "time::serde::rfc3339::option")]
    pub deactivated_at: Option<time::OffsetDateTime>,
}

impl ServerUserResponse {
    /// Builds a response from a domain user.
    #[must_use]
    pub fn from_domain(user: &ruckchat_domain::User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            is_server_admin: user.is_server_admin,
            deactivated_at: user.deactivated_at,
        }
    }
}

/// Request to rename an organization.
#[derive(Debug, Clone, Deserialize)]
pub struct RenameOrganizationRequest {
    /// New organization display name.
    pub name: String,
}

/// Response returned after resetting a password.
#[derive(Debug, Clone, Serialize)]
pub struct ResetPasswordResponse {
    /// New temporary password.
    pub password: String,
}

/// Request to create a user as a server administrator.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServerUserRequest {
    /// Email address for the new account.
    pub email: String,
    /// Display name for the new account.
    pub display_name: String,
    /// Optional initial password. When omitted, a temporary password is generated.
    pub password: Option<String>,
}

/// Response returned after creating a user as a server administrator.
#[derive(Debug, Clone, Serialize)]
pub struct CreateServerUserResponse {
    /// The newly created user.
    pub user: ServerUserResponse,
    /// Plain initial password, either supplied or generated.
    pub password: String,
}

/// Request to update a user as a server administrator.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerUserRequest {
    /// New display name, if changing.
    pub display_name: Option<String>,
    /// New avatar URL, if changing.
    pub avatar_url: Option<String>,
    /// New email address, if changing.
    pub email: Option<String>,
}

/// Request to update server-wide settings.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServerSettingsRequest {
    /// Whether maintenance mode is enabled.
    pub maintenance_mode_enabled: bool,
    /// Default maximum file upload size in bytes.
    pub default_max_file_size_bytes: i64,
    /// Default storage quota in bytes.
    pub default_storage_quota_bytes: i64,
    /// Allowed email domains for signup.
    pub allowed_signup_domains: Vec<String>,
    /// Whether new user registrations are allowed.
    pub allow_registration: bool,
    /// Whether the server-side spell checker is enabled.
    pub spelling_enabled: bool,
    /// Default language tag for the spell checker.
    pub spelling_default_language: String,
}

/// Request to start an impersonation session.
#[derive(Debug, Clone, Deserialize)]
pub struct ImpersonateRequest {
    /// User to impersonate.
    pub target_user_id: Uuid,
}

/// Response containing the impersonation session token.
#[derive(Debug, Clone, Serialize)]
pub struct ImpersonateResponse {
    /// Plain impersonation session token.
    pub token: String,
}

/// Request to end an impersonation session.
#[derive(Debug, Clone, Deserialize)]
pub struct EndImpersonateRequest {
    /// Impersonation session token to invalidate.
    pub token: String,
}
