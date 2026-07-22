//! Authentication route handlers and extractor.

use crate::{
    Error,
    handlers::dto::{LoginResponse, RegisterResponse, UserResponse},
    services::dto::{LoginRequest, RegisterRequest},
    state::AppState,
};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE, request::Parts},
    response::{IntoResponse, Response},
};
use ruckchat_id::UserId;

/// Authenticated user extracted from the session cookie or bearer token.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Authenticated user identifier.
    pub id: UserId,
    /// Plain session token used to authenticate the request.
    pub token: String,
}

impl AuthUser {
    /// Hashes the session token the same way the auth service does.
    #[must_use]
    pub fn token_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts)
            .ok_or_else(|| unauthorized_response("missing session cookie or bearer token"))?;
        let user_id = state
            .auth
            .authenticate(&token)
            .await
            .map_err(|err| err.into_response())?;
        Ok(Self { id: user_id, token })
    }
}

fn extract_token(parts: &Parts) -> Option<String> {
    if let Some(auth) = parts.headers.get("authorization")
        && let Ok(auth) = auth.to_str()
        && let Some(token) = auth.strip_prefix("Bearer ")
    {
        return Some(token.trim().to_string());
    }
    if let Some(cookie) = parts.headers.get("cookie")
        && let Ok(cookie) = cookie.to_str()
    {
        for pair in cookie.split(';') {
            let mut kv = pair.trim().splitn(2, '=');
            if let (Some(name), Some(value)) = (kv.next(), kv.next())
                && name == "ruckchat_session"
            {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn unauthorized_response(message: &str) -> Response {
    use crate::handlers::error::ErrorBody;
    let body = ErrorBody {
        code: "unauthorized",
        error: message.into(),
    };
    (StatusCode::UNAUTHORIZED, Json(body)).into_response()
}

/// Registers a new user and their initial organization.
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, Error> {
    let (user, organization) = state.auth.register(request).await?;
    let response = RegisterResponse {
        user: UserResponse::from_domain(&user),
        organization,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

/// Authenticates a user and establishes a session cookie.
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, Error> {
    let login = state.auth.login(request).await?;
    let user = state.users.get_profile(login.user_id).await?;
    let cookie = session_cookie(&login.token, state.environment_secure());
    let response = LoginResponse {
        token: login.token,
        user: UserResponse::from_domain(&user),
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().expect("valid cookie header value"),
    );
    Ok((StatusCode::OK, headers, Json(response)))
}

/// Invalidates the current session.
pub async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, Error> {
    state.auth.logout(&auth_user.token).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn session_cookie(token: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "ruckchat_session={token}; HttpOnly{secure_flag}; SameSite=Strict; Path=/; Max-Age=2592000"
    )
}
