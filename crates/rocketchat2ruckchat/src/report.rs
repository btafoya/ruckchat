//! Dry-run / applied-run JSON report generation.

use std::path::PathBuf;

use serde::Serialize;

use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::ruckchat::client::MigrationData;
use crate::ruckchat::models::ImportResponse;

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
}

/// Per-category counts from the produced snapshot.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SnapshotCounts {
    users: usize,
    organizations: usize,
    memberships: usize,
    roles: usize,
    permissions: usize,
    grants: usize,
    teams: usize,
    team_memberships: usize,
    channels: usize,
    channel_memberships: usize,
    direct_messages: usize,
    messages: usize,
    reactions: usize,
    files: usize,
    emoji: usize,
}

impl Report {
    /// Builds a report from the run configuration, snapshot, and import response.
    #[must_use]
    pub fn from_run(
        config: &ResolvedConfig,
        data: &MigrationData,
        response: ImportResponse,
    ) -> Self {
        Self {
            dry_run: config.is_dry_run(),
            apply: config.apply,
            inserted: response.inserted,
            skipped: response.skipped,
            counts: SnapshotCounts {
                users: data.users.len(),
                organizations: data.organizations.len(),
                memberships: data.organization_memberships.len(),
                roles: data.organization_roles.len(),
                permissions: data.permissions.len(),
                grants: data.role_permissions.len(),
                teams: data.teams.len(),
                team_memberships: data.team_memberships.len(),
                channels: data.channels.len(),
                channel_memberships: data.channel_memberships.len(),
                direct_messages: data.direct_message_conversations.len(),
                messages: data.messages.len(),
                reactions: data.reactions.len(),
                files: data.files.len(),
                emoji: data.custom_emoji.len(),
            },
        }
    }
}

/// Writes a report to a JSON file next to the mapping store.
///
/// The default file name is `<mapping-store-stem>.report.json`.
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
pub fn write_report(config: &ResolvedConfig, report: &Report) -> Result<PathBuf> {
    write(config, report)
}
