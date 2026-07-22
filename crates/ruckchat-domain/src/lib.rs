//! Domain entities, value objects, and repository interfaces for RuckChat.
//!
//! This crate contains the core domain model used by the server service and
//! repository layers. It deliberately avoids infrastructure concerns such as
//! SQLx, HTTP, or file storage backends.

pub mod channel;
pub mod channel_membership;
pub mod direct_message_conversation;
pub mod file;
pub mod message;
pub mod organization;
pub mod organization_membership;
pub mod organization_settings;
pub mod reaction;
pub mod repositories;
pub mod role;
pub mod session;
pub mod user;

pub use channel::Channel;
pub use channel_membership::ChannelMembership;
pub use direct_message_conversation::DirectMessageConversation;
pub use file::File;
pub use message::{ConversationType, Message};
pub use organization::Organization;
pub use organization_membership::OrganizationMembership;
pub use organization_settings::OrganizationSettings;
pub use reaction::Reaction;
pub use repositories::{
    ChannelMembershipRepository, ChannelRepository, DirectMessageConversationRepository,
    FileRepository, MessageRepository, OrganizationMembershipRepository, OrganizationRepository,
    OrganizationSettingsRepository, ReactionRepository, SessionRepository, UserRepository,
};
pub use role::Role;
pub use session::Session;
pub use user::User;
