//! Shared application state.
//!
//! This state is passed to HTTP handlers. It holds the database pool and the
//! service layer backed by SQLx repository implementations.

use crate::{
    mcp::McpHttpService,
    plugins::{
        CompositeEventBus,
        manager::{PluginManager, PluginManagerDeps},
    },
    repositories::{
        AuditLogRepositorySqlx, ChannelMembershipRepositorySqlx, ChannelRepositorySqlx,
        CustomEmojiRepositorySqlx, DirectMessageConversationRepositorySqlx, FileRepositorySqlx,
        MessageRepositorySqlx, OrganizationMembershipRepositorySqlx, OrganizationRepositorySqlx,
        OrganizationRoleRepositorySqlx, OrganizationSettingsRepositorySqlx,
        PermissionRepositorySqlx, ReactionRepositorySqlx, ServerSettingsRepositorySqlx,
        SessionRepositorySqlx, TeamRepositorySqlx, UserRepositorySqlx,
        WebPushSubscriptionRepositorySqlx,
    },
    services::{
        admin::{AdminService, AdminServiceDeps},
        audit::{AuditService, AuditServiceDeps},
        auth::{AuthService, AuthServiceDeps},
        authorization::AuthorizationService,
        channel::{ChannelService, ChannelServiceDeps},
        direct_message::{DirectMessageService, DirectMessageServiceDeps},
        file::{FileService, FileServiceDeps},
        mcp::{McpService, McpServiceDeps},
        message::{MessageService, MessageServiceDeps},
        organization::{OrganizationService, OrganizationServiceDeps},
        reaction::{ReactionService, ReactionServiceDeps},
        server_admin::{ServerAdminService, ServerAdminServiceDeps},
        server_settings::{
            ServerSettingsOverride, ServerSettingsService, ServerSettingsServiceDeps,
        },
        user::{UserService, UserServiceDeps},
        web_push::{WebPushService, WebPushServiceConfig, WebPushServiceDeps},
    },
    websocket::{ConnectionManager, WebSocketEventBus, WebSocketEventBusDeps},
};
use sqlx::PgPool;
use std::sync::Arc;

/// Application state shared across HTTP handlers and background tasks.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub pool: PgPool,
    /// Whether to mark session cookies as `Secure`.
    pub secure_cookies: bool,
    /// Whether the MCP endpoint is enabled.
    pub mcp_enabled: bool,
    /// Whether MCP `post_message` requires explicit confirmation.
    pub mcp_require_confirmation: bool,
    /// Directory containing plugin dynamic libraries.
    pub plugin_dir: String,
    /// Authentication service.
    pub auth: AuthService,
    /// User service.
    pub users: UserService,
    /// Organization service.
    pub organizations: OrganizationService,
    /// Channel service.
    pub channels: ChannelService,
    /// Message service.
    pub messages: MessageService,
    /// Direct message service.
    pub direct_messages: DirectMessageService,
    /// File service.
    pub files: FileService,
    /// Reaction service.
    pub reactions: ReactionService,
    /// Authorization service.
    pub authorization: AuthorizationService,
    /// WebSocket connection manager.
    pub websocket_manager: ConnectionManager,
    /// Plugin manager.
    pub plugin_manager: Arc<PluginManager>,
    /// WebSocket + plugin event bus.
    pub events: CompositeEventBus,
    /// MCP service.
    pub mcp: McpService,
    /// MCP Streamable HTTP service.
    pub mcp_http: McpHttpService,
    /// Web UI configuration.
    pub web_config: ruckchat_config::WebConfig,
    /// Web Push notification service, if enabled and configured.
    pub web_push: Option<WebPushService>,
    /// Administrative service for imports and organization metadata.
    pub admin: AdminService,
    /// Server-wide administrative service.
    pub server_admin: ServerAdminService,
    /// Server-wide settings service.
    pub server_settings: ServerSettingsService,
    /// Audit log service.
    pub audit: AuditService,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("pool", &self.pool)
            .field("secure_cookies", &self.secure_cookies)
            .field("mcp_enabled", &self.mcp_enabled)
            .field("mcp_require_confirmation", &self.mcp_require_confirmation)
            .field("plugin_dir", &self.plugin_dir)
            .field("websocket_manager", &self.websocket_manager)
            .field("plugin_manager", &"PluginManager")
            .field("mcp", &"McpService")
            .field("mcp_http", &"McpHttpService")
            .field(
                "web_push",
                &self.web_push.as_ref().map(|_| "WebPushService"),
            )
            .field("admin", &"AdminService")
            .field("server_admin", &"ServerAdminService")
            .field("server_settings", &"ServerSettingsService")
            .field("audit", &"AuditService")
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Creates state from a connection pool, building services backed by SQLx
    /// repositories.
    #[must_use]
    pub fn from_pool(
        pool: PgPool,
        secure_cookies: bool,
        mcp_enabled: bool,
        mcp_require_confirmation: bool,
        plugin_dir: String,
        files_directory: String,
    ) -> Self {
        Self::build(
            pool,
            secure_cookies,
            mcp_enabled,
            mcp_require_confirmation,
            plugin_dir,
            ruckchat_config::WebConfig::default(),
            &ruckchat_config::WebPushConfig::default(),
            &files_directory,
            ServerSettingsOverride::default(),
        )
    }

    /// Creates state from a loaded [`AppConfig`] and an existing connection pool.
    #[must_use]
    pub fn from_config(pool: PgPool, config: &ruckchat_config::AppConfig) -> Self {
        let secure_cookies = matches!(config.environment, ruckchat_config::Environment::Production);
        let server_settings_overrides = ServerSettingsOverride {
            maintenance_mode_enabled: config.server_settings.maintenance_mode_enabled,
            default_max_file_size_bytes: config.server_settings.default_max_file_size_bytes,
            default_storage_quota_bytes: config.server_settings.default_storage_quota_bytes,
            allowed_signup_domains: config.server_settings.allowed_signup_domains.clone(),
            allow_registration: config.server_settings.allow_registration,
        };
        Self::build(
            pool,
            secure_cookies,
            config.mcp.enabled,
            config.mcp.require_confirmation,
            config.plugins.directory.clone(),
            config.web.clone(),
            &config.web_push,
            &config.files.directory,
            server_settings_overrides,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        pool: PgPool,
        secure_cookies: bool,
        mcp_enabled: bool,
        mcp_require_confirmation: bool,
        plugin_dir: String,
        web_config: ruckchat_config::WebConfig,
        web_push_config: &ruckchat_config::WebPushConfig,
        files_directory: &str,
        server_settings_overrides: ServerSettingsOverride,
    ) -> Self {
        let users_repo = Arc::new(UserRepositorySqlx::new(pool.clone()));
        let sessions_repo = Arc::new(SessionRepositorySqlx::new(pool.clone()));
        let organizations_repo = Arc::new(OrganizationRepositorySqlx::new(pool.clone()));
        let memberships_repo = Arc::new(OrganizationMembershipRepositorySqlx::new(pool.clone()));
        let settings_repo = Arc::new(OrganizationSettingsRepositorySqlx::new(pool.clone()));
        let channels_repo = Arc::new(ChannelRepositorySqlx::new(pool.clone()));
        let channel_memberships_repo = Arc::new(ChannelMembershipRepositorySqlx::new(pool.clone()));
        let conversations_repo =
            Arc::new(DirectMessageConversationRepositorySqlx::new(pool.clone()));
        let messages_repo = Arc::new(MessageRepositorySqlx::new(pool.clone()));
        let reactions_repo = Arc::new(ReactionRepositorySqlx::new(pool.clone()));
        let files_repo = Arc::new(FileRepositorySqlx::new(pool.clone()));
        let web_push_subscriptions_repo =
            Arc::new(WebPushSubscriptionRepositorySqlx::new(pool.clone()));
        let organization_roles_repo = Arc::new(OrganizationRoleRepositorySqlx::new(pool.clone()));
        let permissions_repo = Arc::new(PermissionRepositorySqlx::new(pool.clone()));
        let custom_emoji_repo = Arc::new(CustomEmojiRepositorySqlx::new(pool.clone()));
        let teams_repo = Arc::new(TeamRepositorySqlx::new(pool.clone()));
        let server_settings_repo = Arc::new(ServerSettingsRepositorySqlx::new(pool.clone()));
        let audit_log_repo = Arc::new(AuditLogRepositorySqlx::new(pool.clone()));

        let web_push = WebPushServiceConfig::from_config(web_push_config)
            .and_then(|svc_config| {
                WebPushService::new(
                    WebPushServiceDeps {
                        subscriptions: web_push_subscriptions_repo.clone(),
                        conversations: conversations_repo.clone(),
                        channel_memberships: channel_memberships_repo.clone(),
                        users: users_repo.clone(),
                    },
                    svc_config,
                )
                .map_err(|err| {
                    tracing::warn!(%err, "failed to initialize web push service; continuing without push notifications");
                    err
                })
                .ok()
            });

        let authorization = AuthorizationService::new();
        let connection_manager = ConnectionManager::new();
        let websocket_events = WebSocketEventBus::new(WebSocketEventBusDeps {
            manager: connection_manager.clone(),
            messages: messages_repo.clone(),
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships_repo.clone(),
            conversations: conversations_repo.clone(),
            memberships: memberships_repo.clone(),
        });

        let plugin_manager = Arc::new(
            PluginManager::load_from_dir(
                std::path::Path::new(&plugin_dir),
                PluginManagerDeps {
                    users: users_repo.clone(),
                    channels: channels_repo.clone(),
                    messages: messages_repo.clone(),
                    events: Arc::new(websocket_events.clone()),
                },
            )
            .unwrap_or_else(|err| {
                tracing::warn!(%err, plugin_dir = %plugin_dir, "failed to load plugins; continuing without plugins");
                PluginManager::empty()
            }),
        );

        let events =
            CompositeEventBus::new(websocket_events, plugin_manager.clone(), web_push.clone());

        let auth = AuthService::new(AuthServiceDeps {
            users: users_repo.clone(),
            sessions: sessions_repo.clone(),
            organizations: organizations_repo.clone(),
            memberships: memberships_repo.clone(),
            settings: settings_repo.clone(),
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships_repo.clone(),
        });

        let users = UserService::new(UserServiceDeps {
            users: users_repo.clone(),
            memberships: memberships_repo.clone(),
        });

        let organizations = OrganizationService::new(OrganizationServiceDeps {
            organizations: organizations_repo.clone(),
            users: users_repo.clone(),
            memberships: memberships_repo.clone(),
            settings: settings_repo.clone(),
            authorization: authorization.clone(),
        });

        let channels = ChannelService::new(ChannelServiceDeps {
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships_repo.clone(),
            memberships: memberships_repo.clone(),
            authorization: authorization.clone(),
        });

        let messages = MessageService::new(MessageServiceDeps {
            messages: messages_repo.clone(),
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships_repo.clone(),
            memberships: memberships_repo.clone(),
            conversations: conversations_repo.clone(),
            users: users_repo.clone(),
            authorization: authorization.clone(),
            events: Arc::new(events.clone()),
        });

        let reactions = ReactionService::new(ReactionServiceDeps {
            reactions: reactions_repo.clone(),
            messages: messages_repo.clone(),
            channels: channels_repo.clone(),
            channel_memberships: channel_memberships_repo.clone(),
            memberships: memberships_repo.clone(),
            conversations: conversations_repo.clone(),
            events: Arc::new(events.clone()),
        });

        let direct_messages = DirectMessageService::new(DirectMessageServiceDeps {
            conversations: conversations_repo.clone(),
            memberships: memberships_repo.clone(),
        });

        let files = FileService::new(
            FileServiceDeps {
                files: files_repo.clone(),
                messages: messages_repo.clone(),
                memberships: memberships_repo.clone(),
                settings: settings_repo.clone(),
            },
            files_directory,
        );

        let mcp = McpService::new(
            McpServiceDeps {
                channels: channels.clone(),
                direct_messages: direct_messages.clone(),
                messages: messages.clone(),
                users: users.clone(),
                organizations: organizations.clone(),
                memberships: memberships_repo.clone(),
            },
            mcp_require_confirmation,
        );
        let mcp_http = McpHttpService::new(mcp.clone());

        let admin = AdminService::new(AdminServiceDeps {
            pool: pool.clone(),
            organizations: organizations_repo.clone(),
            users: users_repo.clone(),
            memberships: memberships_repo.clone(),
            roles: organization_roles_repo.clone(),
            permissions: permissions_repo.clone(),
            emoji: custom_emoji_repo.clone(),
            teams: teams_repo.clone(),
            organization_settings: settings_repo.clone(),
            files: files_repo.clone(),
        });

        let audit = AuditService::new(AuditServiceDeps {
            audit_log: audit_log_repo.clone(),
        });

        let server_settings = ServerSettingsService::new(ServerSettingsServiceDeps {
            repository: server_settings_repo.clone(),
            overrides: server_settings_overrides,
        });

        let server_admin = ServerAdminService::new(ServerAdminServiceDeps {
            users: users_repo.clone(),
            organizations: organizations_repo.clone(),
            memberships: memberships_repo.clone(),
            organization_settings: settings_repo.clone(),
            server_settings: server_settings_repo.clone(),
            auth: auth.clone(),
            audit: audit.clone(),
        });

        Self {
            pool,
            secure_cookies,
            mcp_enabled,
            mcp_require_confirmation,
            plugin_dir,
            auth,
            users,
            organizations,
            channels,
            messages,
            direct_messages,
            files,
            reactions,
            authorization,
            websocket_manager: connection_manager,
            plugin_manager,
            events,
            mcp,
            mcp_http,
            web_config,
            web_push,
            admin,
            server_admin,
            server_settings,
            audit,
        }
    }

    /// Returns true when cookies should be marked `Secure`.
    #[must_use]
    pub fn environment_secure(&self) -> bool {
        self.secure_cookies
    }
}
