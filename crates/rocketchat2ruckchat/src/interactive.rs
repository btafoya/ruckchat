//! Interactive prompts for missing configuration values.

use std::path::PathBuf;

use dialoguer::{Confirm, Input, Password, Select};

use crate::config::{
    RocketAuthConfig, RocketLoginAuth, RocketPatAuth, RuckAuthConfig, SourceConfig, TargetConfig,
};
use crate::error::{Error, Result};

/// Prompts for source configuration if it is incomplete.
pub fn prompt_source(source: &mut Option<SourceConfig>) -> Result<()> {
    let mut url = source.as_ref().map(|s| s.url.clone()).unwrap_or_default();
    if url.is_empty() {
        url = Input::new()
            .with_prompt("RocketChat URL")
            .default("https://rocketchat.example.com".into())
            .interact_text()?;
    }

    warn_if_insecure(&url);

    let auth = source.as_ref().map(|s| s.auth.clone()).unwrap_or_default();
    let auth = prompt_rocket_auth(auth)?;

    *source = Some(SourceConfig { url, auth });
    Ok(())
}

/// Prompts for target configuration if it is incomplete.
pub fn prompt_target(target: &mut Option<TargetConfig>) -> Result<()> {
    let mut url = target.as_ref().map(|t| t.url.clone()).unwrap_or_default();
    if url.is_empty() {
        url = Input::new()
            .with_prompt("RuckChat URL")
            .default("http://localhost:3000".into())
            .interact_text()?;
    }

    warn_if_insecure(&url);

    let mut organization_id = target
        .as_ref()
        .map(|t| t.organization_id.to_string())
        .unwrap_or_default();
    if organization_id.is_empty() || organization_id.parse::<uuid::Uuid>().is_err() {
        organization_id = Input::new()
            .with_prompt("Target organization ID")
            .validate_with(|input: &String| {
                input
                    .parse::<uuid::Uuid>()
                    .map(|_| ())
                    .map_err(|e| format!("invalid UUID: {e}"))
            })
            .interact_text()?;
    }

    let auth = target.as_ref().map(|t| t.auth.clone()).unwrap_or_default();
    let auth = prompt_ruck_auth(auth)?;

    *target = Some(TargetConfig {
        url,
        auth,
        organization_id: organization_id.parse().expect("validated UUID"),
    });
    Ok(())
}

/// Prompts for the SQLite mapping store path.
pub fn prompt_mapping_store(default: PathBuf) -> Result<PathBuf> {
    let path: String = Input::new()
        .with_prompt("Mapping store path")
        .default(default.to_string_lossy().into_owned())
        .interact_text()?;
    Ok(PathBuf::from(path))
}

/// Prompts for source authentication, preferring PAT when available.
fn prompt_rocket_auth(mut auth: RocketAuthConfig) -> Result<RocketAuthConfig> {
    let has_pat = auth.pat.as_ref().is_some_and(|p| !p.auth_token.is_empty());
    let has_login = auth
        .login
        .as_ref()
        .is_some_and(|l| !l.username.is_empty() && !l.password.is_empty());

    if !has_pat && !has_login {
        let choices = vec!["Personal access token", "Username/password"];
        let idx = Select::new()
            .with_prompt("RocketChat authentication method")
            .items(&choices)
            .default(0)
            .interact()?;

        if idx == 0 {
            let user_id = Input::new()
                .with_prompt("RocketChat user ID")
                .interact_text()?;
            let auth_token = Password::new()
                .with_prompt("RocketChat personal access token")
                .interact()?;
            auth.pat = Some(RocketPatAuth {
                user_id,
                auth_token,
            });
        } else {
            let username = Input::new()
                .with_prompt("RocketChat username")
                .interact_text()?;
            let password = Password::new()
                .with_prompt("RocketChat password")
                .interact()?;
            auth.login = Some(RocketLoginAuth { username, password });
        }
    }

    Ok(auth)
}

/// Prompts for target credentials if they are incomplete.
fn prompt_ruck_auth(mut auth: RuckAuthConfig) -> Result<RuckAuthConfig> {
    if auth.login.email.is_empty() {
        auth.login.email = Input::new().with_prompt("RuckChat email").interact_text()?;
    }
    if auth.login.password.is_empty() {
        auth.login.password = Password::new()
            .with_prompt("RuckChat password")
            .interact()?;
    }
    Ok(auth)
}

/// Asks for confirmation before applying a migration.
pub fn confirm_apply() -> Result<()> {
    let confirmed = Confirm::new()
        .with_prompt("This will write data to RuckChat. Continue?")
        .default(false)
        .interact()?;
    if !confirmed {
        return Err(Error::Input("apply cancelled".into()));
    }
    Ok(())
}

fn warn_if_insecure(url: &str) {
    if url.starts_with("http://") && !url.starts_with("http://localhost") {
        tracing::warn!("Target URL uses plain HTTP over a network: {url}");
    }
}
