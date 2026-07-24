//! RuckChat REST API client used by the migration tool.

use reqwest::{Client, StatusCode};
use ruckchat_domain::{
    Channel, ChannelMembership, CustomEmoji, DirectMessageConversation, File, Organization,
    OrganizationMembership, OrganizationRole, OrganizationSettings, Permission, Reaction, Team,
    TeamMembership, TeamRoom, User,
};
use ruckchat_id::{FileId, OrganizationId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config::RuckAuthConfig;
use crate::error::{Error, Result};
use crate::ruckchat::auth::authenticate;
use crate::ruckchat::models::{FileResponse, ImportResponse};

/// Version of the migration snapshot format this tool produces.
pub const MIGRATION_VERSION: u16 = 2;

/// Complete snapshot of data to import into RuckChat.
///
/// This mirrors the server's `MigrationData` struct in `server/src/migrate.rs`
/// so the admin import endpoint can deserialize it directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationData {
    /// Format version of this snapshot.
    pub version: u16,
    /// UTC timestamp when the snapshot was produced.
    #[serde(with = "time::serde::rfc3339")]
    pub exported_at: OffsetDateTime,
    /// User accounts.
    pub users: Vec<User>,
    /// Organizations.
    pub organizations: Vec<Organization>,
    /// Organization memberships.
    pub organization_memberships: Vec<OrganizationMembership>,
    /// Per-organization settings.
    pub organization_settings: Vec<OrganizationSettings>,
    /// Custom organization roles.
    pub organization_roles: Vec<OrganizationRole>,
    /// Permissions defined within organizations.
    pub permissions: Vec<Permission>,
    /// Role-permission grants.
    pub role_permissions: Vec<RuckRolePermission>,
    /// Custom emoji.
    pub custom_emoji: Vec<CustomEmoji>,
    /// Teams.
    pub teams: Vec<Team>,
    /// Team memberships.
    pub team_memberships: Vec<TeamMembership>,
    /// Team-room links.
    pub team_rooms: Vec<TeamRoom>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMessage {
    /// Internal message identifier.
    pub id: ruckchat_id::MessageId,
    /// Conversation identifier.
    pub conversation_id: Uuid,
    /// Conversation discriminator matching the database check constraint.
    pub conversation_type: String,
    /// Optional parent message identifier for threads.
    pub parent_id: Option<ruckchat_id::MessageId>,
    /// User who authored the message.
    pub author_id: ruckchat_id::UserId,
    /// Message content.
    pub content: String,
    /// Users explicitly mentioned in the message.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub mentioned_user_ids: Vec<ruckchat_id::UserId>,
    /// Timestamp when the message was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Timestamp of the last edit.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Soft-delete timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
}

/// Role-permission grant for migration snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuckRolePermission {
    /// Role receiving the permission.
    pub role_id: ruckchat_id::OrganizationRoleId,
    /// Permission being granted.
    pub permission_id: ruckchat_id::PermissionId,
}

/// Link between a message and an attached file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFileLink {
    /// Message identifier.
    pub message_id: ruckchat_id::MessageId,
    /// File identifier.
    pub file_id: FileId,
}

/// Client for the RuckChat REST API.
#[derive(Debug, Clone)]
pub struct RuckChatClient {
    http: Client,
    base_url: String,
    token: String,
}

impl RuckChatClient {
    /// Authenticates and creates a new client.
    pub async fn new(base_url: &str, auth: &RuckAuthConfig) -> Result<Self> {
        let http = Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|e| Error::Internal(e.to_string()))?;
        let token = authenticate(&http, base_url, auth).await?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').into(),
            token,
        })
    }

    /// Imports a migration snapshot into the target organization.
    pub async fn import_snapshot(
        &self,
        organization_id: OrganizationId,
        data: &MigrationData,
        dry_run: bool,
    ) -> Result<ImportResponse> {
        let url = format!(
            "{}/api/v1/admin/organizations/{}/import",
            self.base_url,
            organization_id.as_uuid()
        );
        debug!(%url, "RuckChat import snapshot");
        let request_body = serde_json::json!({
            "data": data,
            "dry_run": dry_run,
        });
        self.post_json(&url, request_body).await
    }

    /// Uploads file bytes to RuckChat.
    pub async fn upload_file(
        &self,
        organization_id: OrganizationId,
        file_name: &str,
        mime_type: &str,
        bytes: Vec<u8>,
    ) -> Result<FileResponse> {
        let url = format!("{}/files", self.base_url);
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| Error::Internal(e.to_string()))?;
        let form = reqwest::multipart::Form::new()
            .text("organization_id", organization_id.as_uuid().to_string())
            .part("file", part);

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await?;
        self.parse_response(response).await
    }

    /// Loads full file metadata including the storage path.
    pub async fn get_file_metadata(&self, file_id: FileId) -> Result<ruckchat_domain::File> {
        let url = format!("{}/files/{}", self.base_url, file_id.as_uuid());
        self.get_json(&url).await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.http.get(url).bearer_auth(&self.token).send().await?;
        self.parse_response(response).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<T> {
        debug!(%url, "RuckChat POST");
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;
        self.parse_response(response).await
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            warn!("RuckChat rate limited; backing off");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::ruckchat(
                Some(status),
                format!("RuckChat request failed: {text}"),
            ));
        }
        response.json().await.map_err(Into::into)
    }
}
