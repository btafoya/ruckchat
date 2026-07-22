//! Shared application state.
//!
//! This state is passed to HTTP handlers. It holds the database pool and the
//! service layer backed by SQLx repository implementations.

use crate::{
    repositories::{
        ChannelMembershipRepositorySqlx, ChannelRepositorySqlx,
        DirectMessageConversationRepositorySqlx, FileRepositorySqlx, MessageRepositorySqlx,
        OrganizationMembershipRepositorySqlx, OrganizationRepositorySqlx,
        OrganizationSettingsRepositorySqlx, SessionRepositorySqlx, UserRepositorySqlx,
    },
    services::{
        auth::{AuthService, AuthServiceDeps},
        authorization::AuthorizationService,
        channel::{ChannelService, ChannelServiceDeps},
        direct_message::{DirectMessageService, DirectMessageServiceDeps},
        file::{FileService, FileServiceDeps},
        message::{MessageService, MessageServiceDeps},
        organization::{OrganizationService, OrganizationServiceDeps},
        user::{UserService, UserServiceDeps},
    },
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
    /// Authorization service.
    pub authorization: AuthorizationService,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("pool", &self.pool)
            .field("secure_cookies", &self.secure_cookies)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Creates state from a connection pool, building services backed by SQLx
    /// repositories.
    #[must_use]
    pub fn from_pool(pool: PgPool, secure_cookies: bool) -> Self {
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
        let files_repo = Arc::new(FileRepositorySqlx::new(pool.clone()));

        let authorization = AuthorizationService::new();

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
        });

        let direct_messages = DirectMessageService::new(DirectMessageServiceDeps {
            conversations: conversations_repo.clone(),
            memberships: memberships_repo.clone(),
        });

        let files = FileService::new(FileServiceDeps {
            files: files_repo.clone(),
            messages: messages_repo.clone(),
            memberships: memberships_repo.clone(),
            settings: settings_repo.clone(),
        });

        Self {
            pool,
            secure_cookies,
            auth,
            users,
            organizations,
            channels,
            messages,
            direct_messages,
            files,
            authorization,
        }
    }

    /// Returns true when cookies should be marked `Secure`.
    #[must_use]
    pub fn environment_secure(&self) -> bool {
        self.secure_cookies
    }
}
