//! Data migration export/import for RuckChat.
//!
//! Provides idempotent, versioned export and import of the core domain aggregates
//! stored in PostgreSQL. The format is a single JSON document with a top-level
//! `version` field so future schema changes can be detected.
//!
//! Sessions, Web Push subscriptions, and plugin state are intentionally excluded
//! as runtime/ephemeral data.

use ruckchat_common::time::OffsetDateTime;
use ruckchat_domain::{
    Channel, ChannelMembership, DirectMessageConversation, File, Organization,
    OrganizationMembership, OrganizationSettings, Reaction, User,
};
use ruckchat_id::{FileId, MessageId, UserId};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

/// Current migration format version.
pub const MIGRATION_VERSION: u16 = 1;

/// Result alias for migration operations.
pub type Result<T> = std::result::Result<T, MigrateError>;

/// Errors produced by the migration subsystem.
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Failed to read or write the migration file.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// The migration file has an unsupported version.
    #[error("unsupported migration version: {0}")]
    UnsupportedVersion(u16),

    /// A validation or invariant violation.
    #[error("validation error: {0}")]
    Validation(String),
}

/// Complete snapshot of exportable RuckChat data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationData {
    /// Format version of this snapshot.
    pub version: u16,
    /// UTC timestamp when the snapshot was produced.
    pub exported_at: OffsetDateTime,
    /// User accounts.
    pub users: Vec<User>,
    /// Organizations.
    pub organizations: Vec<Organization>,
    /// Organization memberships.
    pub organization_memberships: Vec<OrganizationMembership>,
    /// Per-organization quotas and limits.
    pub organization_settings: Vec<OrganizationSettings>,
    /// Channels.
    pub channels: Vec<Channel>,
    /// Channel memberships.
    pub channel_memberships: Vec<ChannelMembership>,
    /// Direct message conversations.
    pub direct_message_conversations: Vec<DirectMessageConversation>,
    /// Messages.
    pub messages: Vec<MigrationMessage>,
    /// Message reactions.
    pub reactions: Vec<Reaction>,
    /// File metadata.
    pub files: Vec<File>,
    /// Links between messages and attached files.
    pub message_files: Vec<MessageFileLink>,
}

/// Message record used in migration snapshots.
///
/// Mirrors [`ruckchat_domain::Message`] but stores `conversation_type` as the
/// database string (`"channel"` or `"dm"`) rather than the domain enum's serde
/// representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMessage {
    /// Internal message identifier.
    pub id: MessageId,
    /// Conversation identifier.
    pub conversation_id: Uuid,
    /// Conversation discriminator matching the database check constraint.
    pub conversation_type: String,
    /// Optional parent message identifier for threads.
    pub parent_id: Option<MessageId>,
    /// User who authored the message.
    pub author_id: UserId,
    /// Message content.
    pub content: String,
    /// Timestamp when the message was created.
    pub created_at: OffsetDateTime,
    /// Timestamp of the last edit.
    pub updated_at: OffsetDateTime,
    /// Soft-delete timestamp.
    pub deleted_at: Option<OffsetDateTime>,
}

/// Link between a message and an attached file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFileLink {
    /// Message identifier.
    pub message_id: MessageId,
    /// File identifier.
    pub file_id: FileId,
}

/// Statistics reported by an import operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportCounts {
    /// Rows inserted or updated.
    pub inserted: usize,
    /// Rows skipped because they already existed.
    pub skipped: usize,
}

/// Reads the entire exportable database state into a [`MigrationData`] snapshot.
///
/// # Errors
///
/// Returns [`MigrateError::Database`] when any query fails.
pub async fn export(pool: &PgPool) -> Result<MigrationData> {
    let users = export_users(pool).await?;
    let organizations = export_organizations(pool).await?;
    let organization_memberships = export_organization_memberships(pool).await?;
    let organization_settings = export_organization_settings(pool).await?;
    let channels = export_channels(pool).await?;
    let channel_memberships = export_channel_memberships(pool).await?;
    let direct_message_conversations = export_direct_message_conversations(pool).await?;
    let messages = export_messages(pool).await?;
    let reactions = export_reactions(pool).await?;
    let files = export_files(pool).await?;
    let message_files = export_message_files(pool).await?;

    Ok(MigrationData {
        version: MIGRATION_VERSION,
        exported_at: OffsetDateTime::now_utc(),
        users,
        organizations,
        organization_memberships,
        organization_settings,
        channels,
        channel_memberships,
        direct_message_conversations,
        messages,
        reactions,
        files,
        message_files,
    })
}

/// Writes a migration snapshot to a JSON file.
///
/// # Errors
///
/// Returns [`MigrateError::Json`] on serialization failure or [`MigrateError::Io`]
/// when the file cannot be written.
pub async fn export_to_file(pool: &PgPool, path: impl AsRef<Path>) -> Result<MigrationData> {
    let data = export(pool).await?;
    let json = serde_json::to_string_pretty(&data)?;
    tokio::fs::write(path, json).await?;
    Ok(data)
}

/// Reads a migration snapshot from a JSON file.
///
/// # Errors
///
/// Returns [`MigrateError::Io`] when the file cannot be read, [`MigrateError::Json`]
/// when parsing fails, or [`MigrateError::UnsupportedVersion`] when the version is
/// not recognized.
pub async fn read_migration_file(path: impl AsRef<Path>) -> Result<MigrationData> {
    let content = tokio::fs::read_to_string(path).await?;
    let data: MigrationData = serde_json::from_str(&content)?;
    if data.version != MIGRATION_VERSION {
        return Err(MigrateError::UnsupportedVersion(data.version));
    }
    Ok(data)
}

/// Imports a migration snapshot into the database.
///
/// Existing rows are skipped (`ON CONFLICT DO NOTHING`) so the operation is
/// idempotent. When `dry_run` is true, rows are counted but no data is written.
///
/// # Errors
///
/// Returns [`MigrateError::Database`] when any insert fails, or
/// [`MigrateError::Validation`] when the snapshot references inconsistent data.
pub async fn import(pool: &PgPool, data: &MigrationData, dry_run: bool) -> Result<ImportCounts> {
    if data.version != MIGRATION_VERSION {
        return Err(MigrateError::UnsupportedVersion(data.version));
    }

    validate(data)?;

    if dry_run {
        return Ok(ImportCounts {
            inserted: 0,
            skipped: count_rows(data),
        });
    }

    let mut tx = pool.begin().await?;
    let mut inserted = 0;
    let mut skipped = 0;

    inserted += import_users(&mut tx, &data.users).await?;
    skipped += data.users.len() - inserted;

    let org_inserted = import_organizations(&mut tx, &data.organizations).await?;
    skipped += data.organizations.len() - org_inserted;
    inserted += org_inserted;

    let membership_inserted =
        import_organization_memberships(&mut tx, &data.organization_memberships).await?;
    skipped += data.organization_memberships.len() - membership_inserted;
    inserted += membership_inserted;

    let settings_inserted =
        import_organization_settings(&mut tx, &data.organization_settings).await?;
    skipped += data.organization_settings.len() - settings_inserted;
    inserted += settings_inserted;

    let channel_inserted = import_channels(&mut tx, &data.channels).await?;
    skipped += data.channels.len() - channel_inserted;
    inserted += channel_inserted;

    let channel_membership_inserted =
        import_channel_memberships(&mut tx, &data.channel_memberships).await?;
    skipped += data.channel_memberships.len() - channel_membership_inserted;
    inserted += channel_membership_inserted;

    let dm_inserted =
        import_direct_message_conversations(&mut tx, &data.direct_message_conversations).await?;
    skipped += data.direct_message_conversations.len() - dm_inserted;
    inserted += dm_inserted;

    let message_inserted = import_messages(&mut tx, &data.messages).await?;
    skipped += data.messages.len() - message_inserted;
    inserted += message_inserted;

    let reaction_inserted = import_reactions(&mut tx, &data.reactions).await?;
    skipped += data.reactions.len() - reaction_inserted;
    inserted += reaction_inserted;

    let file_inserted = import_files(&mut tx, &data.files).await?;
    skipped += data.files.len() - file_inserted;
    inserted += file_inserted;

    let message_file_inserted = import_message_files(&mut tx, &data.message_files).await?;
    skipped += data.message_files.len() - message_file_inserted;
    inserted += message_file_inserted;

    tx.commit().await?;

    Ok(ImportCounts { inserted, skipped })
}

fn validate(data: &MigrationData) -> Result<()> {
    let user_ids: std::collections::HashSet<Uuid> =
        data.users.iter().map(|u| u.id.as_uuid()).collect();

    for membership in &data.organization_memberships {
        if !user_ids.contains(&membership.user_id.as_uuid()) {
            return Err(MigrateError::Validation(format!(
                "organization_memberships references missing user {}",
                membership.user_id
            )));
        }
    }

    for channel in &data.channels {
        if !user_ids.contains(&channel.created_by.as_uuid()) {
            return Err(MigrateError::Validation(format!(
                "channel {} references missing creator {}",
                channel.id, channel.created_by
            )));
        }
    }

    for message in &data.messages {
        if !user_ids.contains(&message.author_id.as_uuid()) {
            return Err(MigrateError::Validation(format!(
                "message {} references missing author {}",
                message.id, message.author_id
            )));
        }
        if message.conversation_type != "channel" && message.conversation_type != "dm" {
            return Err(MigrateError::Validation(format!(
                "message {} has invalid conversation_type {}",
                message.id, message.conversation_type
            )));
        }
    }

    Ok(())
}

fn count_rows(data: &MigrationData) -> usize {
    data.users.len()
        + data.organizations.len()
        + data.organization_memberships.len()
        + data.organization_settings.len()
        + data.channels.len()
        + data.channel_memberships.len()
        + data.direct_message_conversations.len()
        + data.messages.len()
        + data.reactions.len()
        + data.files.len()
        + data.message_files.len()
}

async fn export_users(pool: &PgPool) -> Result<Vec<User>> {
    let rows = sqlx::query_as!(
        UserRow,
        "SELECT id, email, display_name, password_hash, avatar_url, created_at, updated_at FROM users ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_user).collect())
}

async fn export_organizations(pool: &PgPool) -> Result<Vec<Organization>> {
    let rows = sqlx::query_as!(
        OrganizationRow,
        "SELECT id, name, slug, owner_id, created_at, updated_at FROM organizations ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_organization).collect())
}

async fn export_organization_memberships(pool: &PgPool) -> Result<Vec<OrganizationMembership>> {
    let rows = sqlx::query_as!(
        OrganizationMembershipRow,
        "SELECT user_id, organization_id, role, joined_at FROM organization_memberships ORDER BY joined_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_organization_membership).collect())
}

async fn export_organization_settings(pool: &PgPool) -> Result<Vec<OrganizationSettings>> {
    let rows = sqlx::query_as!(
        OrganizationSettingsRow,
        "SELECT organization_id, max_file_size_bytes, storage_quota_bytes, updated_at FROM organization_settings ORDER BY organization_id"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_organization_settings).collect())
}

async fn export_channels(pool: &PgPool) -> Result<Vec<Channel>> {
    let rows = sqlx::query_as!(
        ChannelRow,
        "SELECT id, organization_id, name, topic, purpose, is_private, created_by, created_at, archived_at FROM channels ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_channel).collect())
}

async fn export_channel_memberships(pool: &PgPool) -> Result<Vec<ChannelMembership>> {
    let rows = sqlx::query_as!(
        ChannelMembershipRow,
        "SELECT user_id, channel_id, joined_at FROM channel_memberships ORDER BY joined_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_channel_membership).collect())
}

async fn export_direct_message_conversations(
    pool: &PgPool,
) -> Result<Vec<DirectMessageConversation>> {
    let conversation_rows = sqlx::query_as!(
        DirectMessageConversationRow,
        "SELECT id, organization_id, created_at FROM direct_message_conversations ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;

    let mut conversations = Vec::with_capacity(conversation_rows.len());
    for row in conversation_rows {
        let member_uuids = sqlx::query_scalar!(
            "SELECT user_id FROM dm_members WHERE conversation_id = $1 ORDER BY user_id",
            row.id
        )
        .fetch_all(pool)
        .await?;
        conversations.push(into_direct_message_conversation(row, member_uuids));
    }
    Ok(conversations)
}

async fn export_messages(pool: &PgPool) -> Result<Vec<MigrationMessage>> {
    let rows = sqlx::query_as!(
        MessageRow,
        "SELECT id, conversation_id, conversation_type, parent_id, author_id, content, created_at, updated_at, deleted_at FROM messages ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_migration_message).collect())
}

async fn export_reactions(pool: &PgPool) -> Result<Vec<Reaction>> {
    let rows = sqlx::query_as!(
        ReactionRow,
        "SELECT message_id, user_id, emoji, created_at FROM reactions ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_reaction).collect())
}

async fn export_files(pool: &PgPool) -> Result<Vec<File>> {
    let rows = sqlx::query_as!(
        FileRow,
        "SELECT id, organization_id, uploaded_by, file_name, mime_type, size_bytes, storage_path, thumbnail_path, created_at FROM files ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_file).collect())
}

async fn export_message_files(pool: &PgPool) -> Result<Vec<MessageFileLink>> {
    let rows = sqlx::query_as!(
        MessageFileRow,
        "SELECT message_id, file_id FROM message_files ORDER BY message_id, file_id"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(into_message_file_link).collect())
}

async fn import_users(tx: &mut Transaction<'_, Postgres>, users: &[User]) -> Result<usize> {
    let mut inserted = 0;
    for user in users {
        let result = sqlx::query!(
            "INSERT INTO users (id, email, display_name, password_hash, avatar_url, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO NOTHING",
            user.id.as_uuid(),
            user.email,
            user.display_name,
            user.password_hash,
            user.avatar_url,
            user.created_at,
            user.updated_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_organizations(
    tx: &mut Transaction<'_, Postgres>,
    organizations: &[Organization],
) -> Result<usize> {
    let mut inserted = 0;
    for org in organizations {
        let result = sqlx::query!(
            "INSERT INTO organizations (id, name, slug, owner_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO NOTHING",
            org.id.as_uuid(),
            org.name,
            org.slug,
            org.owner_id.as_uuid(),
            org.created_at,
            org.updated_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_organization_memberships(
    tx: &mut Transaction<'_, Postgres>,
    memberships: &[OrganizationMembership],
) -> Result<usize> {
    let mut inserted = 0;
    for membership in memberships {
        let result = sqlx::query!(
            "INSERT INTO organization_memberships (user_id, organization_id, role, joined_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id, organization_id) DO NOTHING",
            membership.user_id.as_uuid(),
            membership.organization_id.as_uuid(),
            membership.role.to_string(),
            membership.joined_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_organization_settings(
    tx: &mut Transaction<'_, Postgres>,
    settings: &[OrganizationSettings],
) -> Result<usize> {
    let mut inserted = 0;
    for setting in settings {
        let result = sqlx::query!(
            "INSERT INTO organization_settings (organization_id, max_file_size_bytes, storage_quota_bytes, updated_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (organization_id) DO NOTHING",
            setting.organization_id.as_uuid(),
            setting.max_file_size_bytes,
            setting.storage_quota_bytes,
            setting.updated_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_channels(
    tx: &mut Transaction<'_, Postgres>,
    channels: &[Channel],
) -> Result<usize> {
    let mut inserted = 0;
    for channel in channels {
        let result = sqlx::query!(
            "INSERT INTO channels (id, organization_id, name, topic, purpose, is_private, is_archived, created_by, created_at, archived_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (id) DO NOTHING",
            channel.id.as_uuid(),
            channel.organization_id.as_uuid(),
            channel.name,
            channel.topic,
            channel.purpose,
            channel.is_private,
            channel.archived_at.is_some(),
            channel.created_by.as_uuid(),
            channel.created_at,
            channel.archived_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_channel_memberships(
    tx: &mut Transaction<'_, Postgres>,
    memberships: &[ChannelMembership],
) -> Result<usize> {
    let mut inserted = 0;
    for membership in memberships {
        let result = sqlx::query!(
            "INSERT INTO channel_memberships (user_id, channel_id, joined_at)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, channel_id) DO NOTHING",
            membership.user_id.as_uuid(),
            membership.channel_id.as_uuid(),
            membership.joined_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_direct_message_conversations(
    tx: &mut Transaction<'_, Postgres>,
    conversations: &[DirectMessageConversation],
) -> Result<usize> {
    let mut inserted = 0;
    for conversation in conversations {
        let result = sqlx::query!(
            "INSERT INTO direct_message_conversations (id, organization_id, created_at)
             VALUES ($1, $2, $3)
             ON CONFLICT (id) DO NOTHING",
            conversation.id.as_uuid(),
            conversation.organization_id.as_uuid(),
            conversation.created_at,
        )
        .execute(&mut **tx)
        .await?;
        if result.rows_affected() > 0 {
            inserted += 1;
            for member_id in &conversation.member_ids {
                sqlx::query!(
                    "INSERT INTO dm_members (conversation_id, user_id)
                     VALUES ($1, $2)
                     ON CONFLICT (conversation_id, user_id) DO NOTHING",
                    conversation.id.as_uuid(),
                    member_id.as_uuid(),
                )
                .execute(&mut **tx)
                .await?;
            }
        }
    }
    Ok(inserted)
}

async fn import_messages(
    tx: &mut Transaction<'_, Postgres>,
    messages: &[MigrationMessage],
) -> Result<usize> {
    let mut inserted = 0;
    for message in messages {
        let result = sqlx::query!(
            "INSERT INTO messages (id, conversation_id, conversation_type, parent_id, author_id, content, created_at, updated_at, deleted_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (id) DO NOTHING",
            message.id.as_uuid(),
            message.conversation_id,
            message.conversation_type,
            message.parent_id.map(|id| id.as_uuid()),
            message.author_id.as_uuid(),
            message.content,
            message.created_at,
            message.updated_at,
            message.deleted_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_reactions(
    tx: &mut Transaction<'_, Postgres>,
    reactions: &[Reaction],
) -> Result<usize> {
    let mut inserted = 0;
    for reaction in reactions {
        let result = sqlx::query!(
            "INSERT INTO reactions (message_id, user_id, emoji, created_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (message_id, user_id, emoji) DO NOTHING",
            reaction.message_id.as_uuid(),
            reaction.user_id.as_uuid(),
            reaction.emoji,
            reaction.created_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_files(tx: &mut Transaction<'_, Postgres>, files: &[File]) -> Result<usize> {
    let mut inserted = 0;
    for file in files {
        let result = sqlx::query!(
            "INSERT INTO files (id, organization_id, uploaded_by, file_name, mime_type, size_bytes, storage_path, thumbnail_path, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (id) DO NOTHING",
            file.id.as_uuid(),
            file.organization_id.as_uuid(),
            file.uploaded_by.as_uuid(),
            file.file_name,
            file.mime_type,
            file.size_bytes,
            file.storage_path,
            file.thumbnail_path,
            file.created_at,
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

async fn import_message_files(
    tx: &mut Transaction<'_, Postgres>,
    links: &[MessageFileLink],
) -> Result<usize> {
    let mut inserted = 0;
    for link in links {
        let result = sqlx::query!(
            "INSERT INTO message_files (message_id, file_id)
             VALUES ($1, $2)
             ON CONFLICT (message_id, file_id) DO NOTHING",
            link.message_id.as_uuid(),
            link.file_id.as_uuid(),
        )
        .execute(&mut **tx)
        .await?;
        inserted += result.rows_affected() as usize;
    }
    Ok(inserted)
}

// Row structs and conversion helpers.

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    display_name: String,
    password_hash: String,
    avatar_url: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn into_user(row: UserRow) -> User {
    User {
        id: UserId::from_uuid(row.id),
        email: row.email,
        display_name: row.display_name,
        password_hash: row.password_hash,
        avatar_url: row.avatar_url,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

#[derive(sqlx::FromRow)]
struct OrganizationRow {
    id: Uuid,
    name: String,
    slug: String,
    owner_id: Uuid,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn into_organization(row: OrganizationRow) -> Organization {
    Organization {
        id: ruckchat_id::OrganizationId::from_uuid(row.id),
        name: row.name,
        slug: row.slug,
        owner_id: UserId::from_uuid(row.owner_id),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

#[derive(sqlx::FromRow)]
struct OrganizationMembershipRow {
    user_id: Uuid,
    organization_id: Uuid,
    role: String,
    joined_at: OffsetDateTime,
}

fn into_organization_membership(row: OrganizationMembershipRow) -> OrganizationMembership {
    OrganizationMembership {
        user_id: UserId::from_uuid(row.user_id),
        organization_id: ruckchat_id::OrganizationId::from_uuid(row.organization_id),
        role: ruckchat_domain::Role::from_str(&row.role).unwrap_or_default(),
        joined_at: row.joined_at,
    }
}

#[derive(sqlx::FromRow)]
struct OrganizationSettingsRow {
    organization_id: Uuid,
    max_file_size_bytes: i64,
    storage_quota_bytes: i64,
    updated_at: OffsetDateTime,
}

fn into_organization_settings(row: OrganizationSettingsRow) -> OrganizationSettings {
    OrganizationSettings {
        organization_id: ruckchat_id::OrganizationId::from_uuid(row.organization_id),
        max_file_size_bytes: row.max_file_size_bytes,
        storage_quota_bytes: row.storage_quota_bytes,
        updated_at: row.updated_at,
    }
}

#[derive(sqlx::FromRow)]
struct ChannelRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    topic: Option<String>,
    purpose: Option<String>,
    is_private: bool,
    created_by: Uuid,
    created_at: OffsetDateTime,
    archived_at: Option<OffsetDateTime>,
}

fn into_channel(row: ChannelRow) -> Channel {
    Channel {
        id: ruckchat_id::ChannelId::from_uuid(row.id),
        organization_id: ruckchat_id::OrganizationId::from_uuid(row.organization_id),
        name: row.name,
        topic: row.topic,
        purpose: row.purpose,
        is_private: row.is_private,
        created_by: UserId::from_uuid(row.created_by),
        created_at: row.created_at,
        archived_at: row.archived_at,
    }
}

#[derive(sqlx::FromRow)]
struct ChannelMembershipRow {
    user_id: Uuid,
    channel_id: Uuid,
    joined_at: OffsetDateTime,
}

fn into_channel_membership(row: ChannelMembershipRow) -> ChannelMembership {
    ChannelMembership {
        user_id: UserId::from_uuid(row.user_id),
        channel_id: ruckchat_id::ChannelId::from_uuid(row.channel_id),
        joined_at: row.joined_at,
    }
}

#[derive(sqlx::FromRow)]
struct DirectMessageConversationRow {
    id: Uuid,
    organization_id: Uuid,
    created_at: OffsetDateTime,
}

fn into_direct_message_conversation(
    row: DirectMessageConversationRow,
    member_uuids: Vec<Uuid>,
) -> DirectMessageConversation {
    DirectMessageConversation {
        id: ruckchat_id::DirectMessageConversationId::from_uuid(row.id),
        organization_id: ruckchat_id::OrganizationId::from_uuid(row.organization_id),
        member_ids: member_uuids.into_iter().map(UserId::from_uuid).collect(),
        created_at: row.created_at,
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    conversation_id: Uuid,
    conversation_type: String,
    parent_id: Option<Uuid>,
    author_id: Uuid,
    content: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
}

fn into_migration_message(row: MessageRow) -> MigrationMessage {
    MigrationMessage {
        id: MessageId::from_uuid(row.id),
        conversation_id: row.conversation_id,
        conversation_type: row.conversation_type,
        parent_id: row.parent_id.map(MessageId::from_uuid),
        author_id: UserId::from_uuid(row.author_id),
        content: row.content,
        created_at: row.created_at,
        updated_at: row.updated_at,
        deleted_at: row.deleted_at,
    }
}

#[derive(sqlx::FromRow)]
struct ReactionRow {
    message_id: Uuid,
    user_id: Uuid,
    emoji: String,
    created_at: OffsetDateTime,
}

fn into_reaction(row: ReactionRow) -> Reaction {
    Reaction {
        message_id: MessageId::from_uuid(row.message_id),
        user_id: UserId::from_uuid(row.user_id),
        emoji: row.emoji,
        created_at: row.created_at,
    }
}

#[derive(sqlx::FromRow)]
struct FileRow {
    id: Uuid,
    organization_id: Uuid,
    uploaded_by: Uuid,
    file_name: String,
    mime_type: String,
    size_bytes: i64,
    storage_path: String,
    thumbnail_path: Option<String>,
    created_at: OffsetDateTime,
}

fn into_file(row: FileRow) -> File {
    File {
        id: FileId::from_uuid(row.id),
        organization_id: ruckchat_id::OrganizationId::from_uuid(row.organization_id),
        uploaded_by: UserId::from_uuid(row.uploaded_by),
        file_name: row.file_name,
        mime_type: row.mime_type,
        size_bytes: row.size_bytes,
        storage_path: row.storage_path,
        thumbnail_path: row.thumbnail_path,
        created_at: row.created_at,
    }
}

#[derive(sqlx::FromRow)]
struct MessageFileRow {
    message_id: Uuid,
    file_id: Uuid,
}

fn into_message_file_link(row: MessageFileRow) -> MessageFileLink {
    MessageFileLink {
        message_id: MessageId::from_uuid(row.message_id),
        file_id: FileId::from_uuid(row.file_id),
    }
}
