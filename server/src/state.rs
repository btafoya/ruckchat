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
        ChannelMembershipRepositorySqlx, ChannelRepositorySqlx,
        DirectMessageConversationRepositorySqlx, FileRepositorySqlx, MessageRepositorySqlx,
        OrganizationMembershipRepositorySqlx, OrganizationRepositorySqlx,
        OrganizationSettingsRepositorySqlx, ReactionRepositorySqlx, SessionRepositorySqlx,
        UserRepositorySqlx, WebPushSubscriptionRepositorySqlx,
    },
    services::{
        auth::{AuthService, AuthServiceDeps},
        authorization::AuthorizationService,
        channel::{ChannelService, ChannelServiceDeps},
        direct_message::{DirectMessageService, DirectMessageServiceDeps},
        file::{FileService, FileServiceDeps},
        mcp::{McpService, McpServiceDeps},
        message::{MessageService, MessageServiceDeps},
        organization::{OrganizationService, OrganizationServiceDeps},
        reaction::{ReactionService, ReactionServiceDeps},
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
        )
    }

    /// Creates state from a loaded [`AppConfig`] and an existing connection pool.
    #[must_use]
    pub fn from_config(pool: PgPool, config: &ruckchat_config::AppConfig) -> Self {
        let secure_cookies = matches!(config.environment, ruckchat_config::Environment::Production);
        Self::build(
            pool,
            secure_cookies,
            config.mcp.enabled,
            config.mcp.require_confirmation,
            config.plugins.directory.clone(),
            config.web.clone(),
            &config.web_push,
            &config.files.directory,
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
        }
    }

    /// Returns true when cookies should be marked `Secure`.
    #[must_use]
    pub fn environment_secure(&self) -> bool {
        self.secure_cookies
    }
}
