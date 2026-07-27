//! Interactive prompts for missing configuration values.

use std::path::PathBuf;

use dialoguer::{Confirm, Input, Password};

use crate::config::{EmailConfig, SourceConfig, TargetConfig};
use crate::error::{Error, Result};

/// Prompts for source configuration if it is incomplete.
pub fn prompt_source(source: &mut Option<SourceConfig>) -> Result<()> {
    let mut mongo_uri = source
        .as_ref()
        .map(|s| s.mongo_uri.clone())
        .unwrap_or_default();
    if mongo_uri.is_empty() {
        mongo_uri = Input::new()
            .with_prompt("Source MongoDB URI (a restored RocketChat dump, not a live instance)")
            .default("mongodb://localhost:27017".into())
            .interact_text()?;
    }

    let mut database = source
        .as_ref()
        .map(|s| s.database.clone())
        .unwrap_or_default();
    if database.is_empty() {
        database = Input::new()
            .with_prompt("Source database name")
            .default("rocketchat".into())
            .interact_text()?;
    }

    *source = Some(SourceConfig {
        mongo_uri,
        database,
    });
    Ok(())
}

/// Prompts for target configuration if it is incomplete.
pub fn prompt_target(target: &mut Option<TargetConfig>) -> Result<()> {
    let mut database_url = target
        .as_ref()
        .map(|t| t.database_url.clone())
        .unwrap_or_default();
    if database_url.is_empty() {
        database_url = Input::new()
            .with_prompt("Target RuckChat PostgreSQL connection string")
            .default("postgres://ruckchat:ruckchat@localhost/ruckchat".into())
            .interact_text()?;
    }

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

    let organization_name = existing_or_prompt(
        target.as_ref().map(|t| t.organization_name.clone()),
        "Organization name (used only if organization_id doesn't already exist)",
        "Migrated Organization",
    )?;
    let organization_slug = existing_or_prompt(
        target.as_ref().map(|t| t.organization_slug.clone()),
        "Organization slug (used only if organization_id doesn't already exist)",
        "migrated",
    )?;
    let admin_email = existing_or_prompt(
        target
            .as_ref()
            .map(|t| t.admin_email.clone())
            .filter(|e| !e.is_empty()),
        "Admin email (matched against an existing target account by email, or created fresh)",
        "",
    )?;
    let upload_directory = existing_or_prompt(
        target
            .as_ref()
            .map(|t| t.upload_directory.clone())
            .filter(|d| !d.is_empty()),
        "Target RuckChat upload directory (must match the target's ruckchat.yaml files.directory)",
        "/var/lib/ruckchat/files",
    )?;

    *target = Some(TargetConfig {
        database_url,
        organization_id: organization_id.parse().expect("validated UUID"),
        organization_name,
        organization_slug,
        admin_email,
        upload_directory,
    });
    Ok(())
}

/// Prompts for Postmark configuration when `--send-emails` was requested but
/// no configuration was supplied.
pub fn prompt_email(email: &mut Option<EmailConfig>) -> Result<()> {
    let server_token = existing_or_prompt(
        email
            .as_ref()
            .map(|e| e.server_token.clone())
            .filter(|t| !t.is_empty()),
        "Postmark server API token",
        "",
    )?;
    let from_address = existing_or_prompt(
        email
            .as_ref()
            .map(|e| e.from_address.clone())
            .filter(|f| !f.is_empty()),
        "From address for credential emails",
        "",
    )?;
    *email = Some(EmailConfig {
        server_token,
        from_address,
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

/// Asks for confirmation before applying a migration.
pub fn confirm_apply() -> Result<()> {
    let confirmed = Confirm::new()
        .with_prompt("This will write data to the target RuckChat database. Continue?")
        .default(false)
        .interact()?;
    if !confirmed {
        return Err(Error::Input("apply cancelled".into()));
    }
    Ok(())
}

fn existing_or_prompt(existing: Option<String>, prompt: &str, default: &str) -> Result<String> {
    if let Some(value) = existing {
        return Ok(value);
    }
    if default.is_empty() {
        // Secrets (tokens) and required-with-no-sane-default fields prompt
        // without a visible default value.
        let looks_secret = prompt.to_ascii_lowercase().contains("token");
        return if looks_secret {
            Ok(Password::new().with_prompt(prompt).interact()?)
        } else {
            Ok(Input::new().with_prompt(prompt).interact_text()?)
        };
    }
    Ok(Input::new()
        .with_prompt(prompt)
        .default(default.into())
        .interact_text()?)
}
