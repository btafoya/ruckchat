//! Service request/response DTOs.
//!
//! These types are intentionally separate from domain entities. They carry the
//! exact fields a service operation needs and can include fields that do not
//! belong to the domain model, such as raw passwords or pagination cursors.

use ruckchat_id::{FileId, MessageId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// Pagination parameters for list operations.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct Pagination {
    /// Maximum number of items to return.
    pub limit: i64,
    /// Number of items to skip.
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

impl Pagination {
    /// Coerces negative values to zero and caps the limit.
    #[must_use]
    pub fn normalized(self) -> Self {
        Self {
            limit: self.limit.clamp(1, 100),
            offset: self.offset.max(0),
        }
    }
}

/// Request to register a new user.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    /// Email address for the new account.
    pub email: String,
    /// Display name shown to other users.
    pub display_name: String,
    /// Plain-text password; hashed by the service.
    pub password: String,
    /// Name for the user's initial organization.
    pub organization_name: String,
    /// URL-safe slug for the initial organization.
    pub organization_slug: String,
}

/// Request to log in.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    /// Account email address.
    pub email: String,
    /// Plain-text password.
    pub password: String,
}

/// Service response for a successful login.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    /// Plain session token to return to the client.
    pub token: String,
    /// User identifier.
    pub user_id: UserId,
}

/// Request to update a user profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateProfileRequest {
    /// New display name, if changing.
    pub display_name: Option<String>,
    /// New avatar URL, if changing.
    pub avatar_url: Option<String>,
}

/// Request to create an organization.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrganizationRequest {
    /// Organization display name.
    pub name: String,
    /// URL-safe unique slug.
    pub slug: String,
}

/// Request to invite a member to an organization.
#[derive(Debug, Clone, Deserialize)]
pub struct InviteMemberRequest {
    /// Email address of the user to invite. In v1 the user must already exist.
    pub email: String,
    /// Role to assign within the organization.
    pub role: ruckchat_domain::Role,
}

/// Request to change a member's role.
#[derive(Debug, Clone, Deserialize)]
pub struct ChangeRoleRequest {
    /// Target user identifier.
    pub user_id: UserId,
    /// New role.
    pub role: ruckchat_domain::Role,
}

/// Request to create a channel.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateChannelRequest {
    /// Channel name unique within the organization.
    pub name: String,
    /// Whether the channel is private.
    pub is_private: bool,
}

/// Request to update a channel.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateChannelRequest {
    /// New topic.
    pub topic: Option<String>,
    /// New purpose.
    pub purpose: Option<String>,
}

/// Request to post a message.
#[derive(Debug, Clone, Deserialize)]
pub struct PostMessageRequest {
    /// Identifier of the channel or DM conversation.
    pub conversation_id: uuid::Uuid,
    /// Type of conversation.
    pub conversation_type: ruckchat_domain::ConversationType,
    /// Optional parent message identifier for thread replies.
    pub parent_id: Option<MessageId>,
    /// Message content.
    pub content: String,
}

/// Request to edit a message.
#[derive(Debug, Clone, Deserialize)]
pub struct EditMessageRequest {
    /// New content.
    pub content: String,
}

/// Request to start a direct message conversation.
#[derive(Debug, Clone, Deserialize)]
pub struct StartDmRequest {
    /// Organization that owns the conversation.
    pub organization_id: OrganizationId,
    /// Other participants. The requesting user is added automatically.
    pub member_ids: Vec<UserId>,
}

/// Request to record a file upload.
#[derive(Debug, Clone, Deserialize)]
pub struct RecordUploadRequest {
    /// Organization that owns the file.
    pub organization_id: OrganizationId,
    /// Original file name.
    pub file_name: String,
    /// MIME type.
    pub mime_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
    /// Storage backend path or key.
    pub storage_path: String,
}

/// Response returned when a file is recorded.
#[derive(Debug, Clone, Serialize)]
pub struct FileResponse {
    /// File identifier.
    pub id: FileId,
    /// Original file name.
    pub file_name: String,
    /// MIME type.
    pub mime_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
}

/// Request to attach a file to a message.
#[derive(Debug, Clone, Deserialize)]
pub struct AttachFileRequest {
    /// Message identifier.
    pub message_id: MessageId,
    /// File identifier.
    pub file_id: FileId,
}
