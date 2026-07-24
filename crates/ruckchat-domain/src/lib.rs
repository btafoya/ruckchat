//! Domain entities, value objects, and repository interfaces for RuckChat.
//!
//! This crate contains the core domain model used by the server service and
//! repository layers. It deliberately avoids infrastructure concerns such as
//! SQLx, HTTP, or file storage backends.

pub mod audit_log;
pub mod channel;
pub mod channel_membership;
pub mod custom_emoji;
pub mod direct_message_conversation;
pub mod file;
pub mod message;
pub mod organization;
pub mod organization_membership;
pub mod organization_role;
pub mod organization_settings;
pub mod permission;
pub mod reaction;
pub mod repositories;
pub mod role;
pub mod role_permission;
pub mod server_settings;
pub mod session;
pub mod team;
pub mod team_membership;
pub mod team_room;
pub mod user;
pub mod web_push_subscription;

pub use audit_log::AuditLogEntry;
pub use channel::Channel;
pub use channel_membership::ChannelMembership;
pub use custom_emoji::CustomEmoji;
pub use direct_message_conversation::DirectMessageConversation;
pub use file::File;
pub use message::{ConversationType, Message};
pub use organization::Organization;
pub use organization_membership::OrganizationMembership;
pub use organization_role::OrganizationRole;
pub use organization_settings::OrganizationSettings;
pub use permission::Permission;
pub use reaction::Reaction;
pub use repositories::{
    AuditLogRepository, ChannelMembershipRepository, ChannelRepository, CustomEmojiRepository,
    DirectMessageConversationRepository, FileRepository, MessageRepository,
    OrganizationMembershipRepository, OrganizationRepository, OrganizationRoleRepository,
    OrganizationSettingsRepository, PermissionRepository, ReactionRepository,
    RolePermissionRepository, ServerSettingsRepository, SessionRepository,
    TeamMembershipRepository, TeamRepository, TeamRoomRepository, UserRepository,
    WebPushSubscriptionRepository,
};
pub use role::Role;
pub use role_permission::OrganizationRolePermission;
pub use server_settings::ServerSettings;
pub use session::Session;
pub use team::Team;
pub use team_membership::{ParseTeamRoleError, TeamMembership, TeamRole};
pub use team_room::TeamRoom;
pub use user::User;
pub use web_push_subscription::WebPushSubscription;
