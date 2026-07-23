//! RuckChat authentication helpers.

use crate::config::RuckAuthConfig;
use crate::error::{Error, Result};
use crate::ruckchat::models::LoginResponse;

use reqwest::Client;

/// Authenticates against RuckChat and returns the session token.
pub async fn authenticate(
    client: &Client,
    base_url: &str,
    auth: &RuckAuthConfig,
) -> Result<String> {
    let url = format!("{}/auth/login", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "email": auth.login.email,
        "password": auth.login.password,
    });

    let response = client.post(&url).json(&body).send().await?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(Error::ruckchat(
            Some(status),
            format!("RuckChat login failed: {text}"),
        ));
    }

    let payload: LoginResponse = response.json().await?;
    Ok(payload.token)
}
