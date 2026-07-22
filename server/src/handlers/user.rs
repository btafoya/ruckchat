//! User route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::UserResponse},
    services::dto::UpdateProfileRequest,
    state::AppState,
};
use axum::{Json, extract::State};

/// Returns the authenticated user's profile.
pub async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>, Error> {
    let user = state.users.get_profile(auth_user.id).await?;
    Ok(Json(UserResponse::from_domain(&user)))
}

/// Updates the authenticated user's profile.
pub async fn update_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, Error> {
    let user = state.users.update_profile(auth_user.id, request).await?;
    Ok(Json(UserResponse::from_domain(&user)))
}
