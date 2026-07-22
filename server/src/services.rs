//! Service layer for RuckChat.
//!
//! Services orchestrate domain aggregates and repository traits to implement
//! use cases. They do not depend on HTTP or WebSocket infrastructure.

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
pub mod user;

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
pub use user::UserService;
