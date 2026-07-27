//! Dry-run / applied-run JSON report generation.

use std::path::PathBuf;

use ruckchat_migrate::{ImportCounts, MigrationData};
use serde::Serialize;

use crate::config::ResolvedConfig;
use crate::error::Result;

/// Summary of a migration run.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Report {
    /// Whether this report reflects a dry run.
    pub dry_run: bool,
    /// Whether `--apply` was supplied.
    pub apply: bool,
    /// Number of rows that would be / were inserted.
    pub inserted: usize,
    /// Number of rows that would be / were skipped.
    pub skipped: usize,
    /// Snapshot row counts by category.
    pub counts: SnapshotCounts,
    /// Outcome of each migration-credential email send attempted, if
    /// `--send-emails` was supplied.
    pub credential_emails: CredentialEmailSummary,
}

/// Per-category counts from the produced snapshot.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SnapshotCounts {
    users: usize,
    organizations: usize,
    memberships: usize,
    channels: usize,
    channel_memberships: usize,
    direct_messages: usize,
    messages: usize,
    reactions: usize,
    files: usize,
    emoji: usize,
}

/// Outcome of a single migration-credential email send.
#[derive(Debug, Clone, Serialize)]
pub struct CredentialEmailOutcome {
    /// Recipient address.
    pub email: String,
    /// Error message, if the send failed.
    pub error: Option<String>,
}

/// Summary of every migration-credential email send attempted.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CredentialEmailSummary {
    /// Number of emails sent successfully.
    pub sent: usize,
    /// Recipients whose send failed, with the error message.
    pub failed: Vec<CredentialEmailOutcome>,
}

impl Report {
    /// Builds a report from the run configuration, snapshot, import result,
    /// and credential-email outcomes.
    #[must_use]
    pub fn from_run(
        config: &ResolvedConfig,
        data: &MigrationData,
        counts: ImportCounts,
        credential_emails: Vec<CredentialEmailOutcome>,
    ) -> Self {
        let (sent, failed): (Vec<_>, Vec<_>) = credential_emails
            .into_iter()
            .partition(|o| o.error.is_none());

        Self {
            dry_run: config.is_dry_run(),
            apply: config.apply,
            inserted: counts.inserted,
            skipped: counts.skipped,
            counts: SnapshotCounts {
                users: data.users.len(),
                organizations: data.organizations.len(),
                memberships: data.organization_memberships.len(),
                channels: data.channels.len(),
                channel_memberships: data.channel_memberships.len(),
                direct_messages: data.direct_message_conversations.len(),
                messages: data.messages.len(),
                reactions: data.reactions.len(),
                files: data.files.len(),
                emoji: data.custom_emoji.len(),
            },
            credential_emails: CredentialEmailSummary {
                sent: sent.len(),
                failed,
            },
        }
    }
}

/// Writes a report to a JSON file next to the mapping store.
///
/// The default file name is `<mapping-store-stem>.report.json`.
///
/// # Errors
///
/// Returns [`crate::error::Error::Io`] when the report cannot be written.
pub fn write(config: &ResolvedConfig, report: &Report) -> Result<PathBuf> {
    let mut path = config.mapping_store.clone();
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "rocketchat2ruckchat".into());
    path.set_file_name(format!("{stem}.report.json"));

    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(&path, json)?;
    Ok(path)
}

/// Writes a report and returns the resulting path.
///
/// # Errors
///
/// Returns [`crate::error::Error::Io`] when the report cannot be written.
pub fn write_report(config: &ResolvedConfig, report: &Report) -> Result<PathBuf> {
    write(config, report)
}
