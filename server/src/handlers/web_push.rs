//! Web Push subscription handlers.

use crate::{Error, handlers::auth::AuthUser, state::AppState};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

/// Response containing the VAPID public key.
#[derive(Debug, Clone, Serialize)]
pub struct VapidKeyResponse {
    /// VAPID public key in URL-safe base64.
    pub public_key: String,
}

/// Subscription request from a browser PushSubscription object.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeRequest {
    /// Push service endpoint URL.
    pub endpoint: String,
    /// P-256 ECDH public key.
    pub p256dh: String,
    /// Authentication secret.
    pub auth: String,
}

/// Unsubscription request identifying the browser subscription to remove.
#[derive(Debug, Clone, Deserialize)]
pub struct UnsubscribeRequest {
    /// Push service endpoint URL.
    pub endpoint: String,
}

/// Returns the configured VAPID public key so the browser can subscribe.
///
/// Returns an empty key string when Web Push is not configured.
pub async fn vapid_key(State(state): State<AppState>) -> Result<Json<VapidKeyResponse>, Error> {
    let public_key = state
        .web_push
        .as_ref()
        .map(|svc| svc.public_key().to_string())
        .unwrap_or_default();
    Ok(Json(VapidKeyResponse { public_key }))
}

/// Stores a browser push subscription for the authenticated user.
pub async fn subscribe(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<SubscribeRequest>,
) -> Result<impl IntoResponse, Error> {
    let Some(web_push) = state.web_push.as_ref() else {
        return Err(ruckchat_common::Error::Forbidden("web push is not enabled".into()).into());
    };
    web_push
        .subscribe(auth_user.id, request.endpoint, request.p256dh, request.auth)
        .await?;
    Ok(StatusCode::CREATED)
}

/// Removes a browser push subscription for the authenticated user.
pub async fn unsubscribe(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<UnsubscribeRequest>,
) -> Result<StatusCode, Error> {
    let Some(web_push) = state.web_push.as_ref() else {
        return Ok(StatusCode::NO_CONTENT);
    };
    web_push.unsubscribe(auth_user.id, request.endpoint).await?;
    Ok(StatusCode::NO_CONTENT)
}
