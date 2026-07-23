//! RocketChat API response models used during migration.

use serde::Deserialize;

/// Wrapper for most RocketChat list responses.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketListResponse<T> {
    /// Whether the call succeeded.
    pub success: bool,
    /// Returned items.
    #[serde(default)]
    pub users: Vec<T>,
    /// Returned channels/rooms.
    #[serde(default)]
    pub channels: Vec<T>,
    /// Returned groups.
    #[serde(default)]
    pub groups: Vec<T>,
    /// Returned IMs.
    #[serde(default)]
    pub ims: Vec<T>,
    /// Returned teams.
    #[serde(default)]
    pub teams: Vec<T>,
    /// Returned emoji.
    #[serde(default)]
    pub emoji: Vec<T>,
    /// Returned roles.
    #[serde(default)]
    pub roles: Vec<T>,
    /// Returned messages.
    #[serde(default)]
    pub messages: Vec<T>,
    /// Total available items.
    #[serde(default)]
    pub total: u64,
    /// Items returned in this page.
    #[serde(default)]
    pub count: u64,
    /// Offset of this page.
    #[serde(default)]
    pub offset: u64,
}

/// RocketChat user record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketUser {
    /// Internal RocketChat identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Display name.
    pub name: Option<String>,
    /// Unique username.
    pub username: String,
    /// Email addresses.
    #[serde(default)]
    pub emails: Vec<RocketEmail>,
    /// Whether the account is active.
    #[serde(default = "default_true")]
    pub active: bool,
    /// Soft-delete timestamp.
    pub deleted_at: Option<String>,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
    /// Optional status message.
    pub status: Option<String>,
}

fn default_true() -> bool {
    true
}

/// RocketChat email entry.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketEmail {
    /// Email address.
    pub address: String,
    /// Whether the address is verified.
    pub verified: bool,
}

/// RocketChat room/channel/group/DM record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketRoom {
    /// Internal RocketChat identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Room type (`c`, `p`, `d`, `l`, `team`/`discussions`).
    #[serde(rename = "t")]
    pub room_type: String,
    /// Room name (for channels/groups).
    pub name: Option<String>,
    /// Friendly name.
    pub fname: Option<String>,
    /// Topic.
    pub topic: Option<String>,
    /// Description/purpose.
    pub description: Option<String>,
    /// Whether the room is archived.
    #[serde(default)]
    pub archived: bool,
    /// Owner user identifier.
    pub u: Option<RocketUserRef>,
    /// Member usernames for DMs.
    #[serde(default)]
    pub usernames: Vec<String>,
    /// Member user identifiers for DMs.
    #[serde(default)]
    pub users: Vec<String>,
    /// Timestamp when the room was created.
    pub ts: Option<String>,
    /// Last-update timestamp.
    pub updated_at: Option<String>,
    /// Optional parent team identifier.
    pub team_id: Option<String>,
}

/// Lightweight user reference embedded in rooms.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketUserRef {
    /// User identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Username.
    pub username: Option<String>,
}

/// RocketChat team record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketTeam {
    /// Team identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Team name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Owner identifier.
    pub owner_id: Option<String>,
    /// Member count.
    pub members: Option<u64>,
    /// Timestamp when the team was created.
    pub created_at: Option<String>,
    /// Timestamp of the last update.
    pub updated_at: Option<String>,
}

/// RocketChat custom emoji record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketEmoji {
    /// Emoji identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Shortcode without colons.
    pub name: String,
    /// File extension.
    pub extension: Option<String>,
    /// Aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// RocketChat role record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketRole {
    /// Role identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Role name.
    pub name: String,
    /// Role scope.
    pub scope: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Optional built-in flag.
    #[serde(default)]
    pub protected: bool,
    /// Granted permissions.
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// RocketChat message record.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketMessage {
    /// Message identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Room identifier.
    #[serde(rename = "rid")]
    pub room_id: String,
    /// Author user identifier.
    #[serde(rename = "u")]
    pub user: RocketUserRef,
    /// Message text.
    #[serde(default)]
    pub msg: String,
    /// Timestamp when the message was created.
    pub ts: Option<String>,
    /// Last-edit timestamp.
    pub edited_at: Option<String>,
    /// Thread parent message identifier.
    pub tmid: Option<String>,
    /// Soft-delete timestamp.
    pub deleted_at: Option<String>,
    /// File attachments.
    #[serde(default)]
    pub attachments: Vec<RocketAttachment>,
    /// Reactions map keyed by emoji shortcode.
    #[serde(default)]
    pub reactions: serde_json::Map<String, serde_json::Value>,
    /// Optional file upload metadata.
    pub file: Option<RocketFile>,
}

/// Attachment embedded in a RocketChat message.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketAttachment {
    /// Display title.
    pub title: Option<String>,
    /// Link to the attachment.
    pub title_link: Option<String>,
    /// MIME type.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// Image URL.
    pub image_url: Option<String>,
    /// Audio URL.
    pub audio_url: Option<String>,
    /// Video URL.
    pub video_url: Option<String>,
    /// File name.
    pub title_link_download: Option<bool>,
    /// Size in bytes.
    pub size: Option<i64>,
}

/// RocketChat file reference inside a message.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RocketFile {
    /// File identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Original file name.
    pub name: Option<String>,
    /// MIME type.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// Size in bytes.
    pub size: Option<i64>,
}

/// RocketChat login response.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketLoginResponse {
    /// Authentication status.
    pub status: String,
    /// Returned data.
    pub data: RocketLoginData,
}

/// Payload of a successful RocketChat login.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketLoginData {
    /// Session token.
    #[serde(rename = "authToken")]
    pub auth_token: String,
    /// User identifier.
    #[serde(rename = "userId")]
    pub user_id: String,
}
