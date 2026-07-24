//! Service layer for RuckChat.
//!
//! Services orchestrate domain aggregates and repository traits to implement
//! use cases. They do not depend on HTTP or WebSocket infrastructure.

pub mod admin;
pub mod audit;
pub mod auth;
pub mod authorization;
pub mod channel;
pub mod direct_message;
pub mod dto;
pub mod events;
pub mod file;
pub mod mcp;
pub mod message;
pub mod organization;
pub mod reaction;
pub mod server_admin;
pub mod server_settings;
pub mod user;
pub mod web_push;

pub use admin::AdminService;
pub use audit::{AuditService, AuditServiceDeps};
pub use auth::AuthService;
pub use authorization::{AuthorizationService, Permission};
pub use channel::ChannelService;
pub use direct_message::DirectMessageService;
pub use events::{ClientMessage, ErrorEvent, EventBus, EventEnvelope, PresenceStatus, ServerEvent};
pub use file::FileService;
pub use mcp::{McpService, McpServiceDeps, PostMessageResult};
pub use message::MessageService;
pub use organization::OrganizationService;
pub use reaction::ReactionService;
pub use server_admin::{ServerAdminService, ServerAdminServiceDeps};
pub use server_settings::{
    ServerSettingsOverride, ServerSettingsService, ServerSettingsServiceDeps,
};
pub use user::UserService;
pub use web_push::{WebPushService, WebPushServiceConfig, WebPushServiceDeps};
