//! HTTP-specific request/response DTOs.
//!
//! These types wrap or extend the service DTOs and domain entities so that the
//! JSON API surface does not leak internal fields such as password hashes.

use ruckchat_domain::Organization;
use ruckchat_id::{MessageId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// Public user representation returned by the API.
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    /// Internal user identifier.
    pub id: UserId,
    /// Globally unique email address.
    pub email: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Optional URL to an avatar image.
    pub avatar_url: Option<String>,
    /// Whether the user is a server-wide administrator.
    pub is_server_admin: bool,
}

impl UserResponse {
    /// Builds a response from a domain user, omitting the password hash.
    #[must_use]
    pub fn from_domain(user: &ruckchat_domain::User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            is_server_admin: user.is_server_admin,
        }
    }
}

/// Response returned on successful registration.
#[derive(Debug, Clone, Serialize)]
pub struct RegisterResponse {
    /// The newly created user.
    pub user: UserResponse,
    /// The user's initial organization.
    pub organization: Organization,
}

/// Response returned on successful login.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    /// Plain session token. The handler also sets this value in an HTTP-only
    /// session cookie for browser clients.
    pub token: String,
    /// Authenticated user.
    pub user: UserResponse,
}

/// Paginated list envelope used by list endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct ListResponse<T> {
    /// Items in the current page.
    pub items: Vec<T>,
}

impl<T> ListResponse<T> {
    /// Wraps a vector of items.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }
}

/// Channel response that includes the channel and a flag indicating whether the
/// caller is a member. Useful for public channel listings.
#[derive(Debug, Clone, Serialize)]
pub struct ChannelMembershipResponse {
    /// Channel identifier.
    pub channel_id: ruckchat_id::ChannelId,
    /// Whether the caller is an explicit member.
    pub is_member: bool,
}

/// Wrapper for endpoints that return a single organization identifier.
#[derive(Debug, Clone, Serialize)]
pub struct OrganizationIdResponse {
    /// Organization identifier.
    pub organization_id: OrganizationId,
}

/// Request to post a message to a channel.
#[derive(Debug, Clone, Deserialize)]
pub struct PostChannelMessageRequest {
    /// Message content.
    pub content: String,
    /// Optional parent message identifier for thread replies.
    pub parent_id: Option<MessageId>,
}

/// Request to post a message to a direct message conversation.
#[derive(Debug, Clone, Deserialize)]
pub struct PostDmMessageRequest {
    /// Message content.
    pub content: String,
    /// Optional parent message identifier for thread replies.
    pub parent_id: Option<MessageId>,
}
