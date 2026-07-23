//! RocketChat authentication helpers.

use crate::config::RocketAuthConfig;
use crate::error::{Error, Result};
use crate::rocket_chat::models::RocketLoginResponse;

use reqwest::Client;

/// Active RocketChat session credentials.
#[derive(Debug, Clone)]
pub struct RocketSession {
    /// Base URL of the RocketChat server.
    pub base_url: String,
    /// Authentication token header value.
    pub auth_token: String,
    /// User identifier header value.
    pub user_id: String,
}

impl RocketSession {
    /// Builds the full URL for an API path.
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.into()
        } else {
            format!("{}{}", self.base_url.trim_end_matches('/'), path)
        }
    }
}

/// Authenticates against RocketChat using PAT or username/password.
pub async fn authenticate(
    client: &Client,
    base_url: &str,
    auth: &RocketAuthConfig,
) -> Result<RocketSession> {
    if let Some(pat) = &auth.pat
        && !pat.auth_token.is_empty()
        && !pat.user_id.is_empty()
    {
        return Ok(RocketSession {
            base_url: base_url.into(),
            auth_token: pat.auth_token.clone(),
            user_id: pat.user_id.clone(),
        });
    }

    let login = auth
        .login
        .as_ref()
        .ok_or_else(|| Error::config("RocketChat username/password or PAT is required"))?;

    let url = format!("{}/api/v1/login", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "username": login.username,
        "password": login.password,
    });

    let response = client.post(&url).json(&body).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(Error::rocketchat(Some(status), "RocketChat login failed"));
    }

    let payload: RocketLoginResponse = response.json().await?;
    if payload.status != "success" {
        return Err(Error::rocketchat(
            None,
            "RocketChat login returned non-success status",
        ));
    }

    Ok(RocketSession {
        base_url: base_url.into(),
        auth_token: payload.data.auth_token,
        user_id: payload.data.user_id,
    })
}
