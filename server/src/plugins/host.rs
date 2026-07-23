//! Server-side implementation of the plugin host API.

use crate::services::events::EventBus;
use ruckchat_domain::{ChannelRepository, MessageRepository, UserRepository};
use ruckchat_id::ChannelId;
use ruckchat_plugin_sdk::{
    Channel, HostApi, LogLevel, Message, PluginEvent, SendMessageRequest, User, UserId,
};
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::{debug, error, info, warn};

/// Server-provided [`HostApi`] used by loaded plugins.
///
/// Each loaded plugin receives its own host handle so that per-plugin
/// configuration and isolation can be enforced in the future.
pub struct ServerHostApi {
    /// Plugin-specific configuration.
    config: serde_json::Value,
    /// User repository for `get_user`.
    users: Arc<dyn UserRepository + Send + Sync>,
    /// Channel repository for `get_channel`.
    channels: Arc<dyn ChannelRepository + Send + Sync>,
    /// Message repository for `send_message`.
    messages: Arc<dyn MessageRepository + Send + Sync>,
    /// Event bus used to publish plugin-originated updates.
    events: Arc<dyn EventBus + Send + Sync>,
    /// Tokio handle for blocking on async services.
    handle: Handle,
}

impl ServerHostApi {
    /// Creates a host API handle for a plugin.
    ///
    /// # Panics
    ///
    /// Panics if called outside of a Tokio runtime.
    #[must_use]
    pub fn new(
        config: serde_json::Value,
        users: Arc<dyn UserRepository + Send + Sync>,
        channels: Arc<dyn ChannelRepository + Send + Sync>,
        messages: Arc<dyn MessageRepository + Send + Sync>,
        events: Arc<dyn EventBus + Send + Sync>,
    ) -> Self {
        Self {
            config,
            users,
            channels,
            messages,
            events,
            handle: Handle::current(),
        }
    }

    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        tokio::task::block_in_place(|| self.handle.block_on(future))
    }
}

impl HostApi for ServerHostApi {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Debug => debug!(message),
            LogLevel::Info => info!(message),
            LogLevel::Warn => warn!(message),
            LogLevel::Error => error!(message),
        }
    }

    fn get_config(&self) -> serde_json::Value {
        self.config.clone()
    }

    fn get_user(&self, user_id: UserId) -> Result<Option<User>, String> {
        self.block_on(async { self.users.by_id(user_id).await })
            .map_err(|err| err.to_string())
    }

    fn get_channel(&self, channel_id: ChannelId) -> Result<Option<Channel>, String> {
        self.block_on(async { self.channels.by_id(channel_id).await })
            .map_err(|err| err.to_string())
    }

    fn send_message(&self, request: SendMessageRequest) -> Result<Message, String> {
        let message = ruckchat_domain::Message::new(
            request.conversation_id,
            request.conversation_type,
            request.author_id,
            request.content,
            request.parent_id,
        )
        .map_err(|err| err.to_string())?;

        self.block_on(async {
            self.messages.create(&message).await?;
            self.events.publish_message_created(&message).await?;
            Ok::<_, ruckchat_common::Error>(message)
        })
        .map_err(|err| err.to_string())
    }

    fn emit_event(&self, event: PluginEvent) -> Result<(), String> {
        match event {
            PluginEvent::MessageReceived { message } => self.block_on(async {
                self.events
                    .publish_message_created(&message)
                    .await
                    .map_err(|err| err.to_string())
            }),
            PluginEvent::MessageUpdated { message } => self.block_on(async {
                self.events
                    .publish_message_updated(&message)
                    .await
                    .map_err(|err| err.to_string())
            }),
            PluginEvent::MessageDeleted { message } => self.block_on(async {
                self.events
                    .publish_message_deleted(&message)
                    .await
                    .map_err(|err| err.to_string())
            }),
            PluginEvent::Notification {
                user_id,
                title,
                body,
            } => {
                info!(%user_id, %title, %body, "plugin notification");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::events::ServerEvent;
    use crate::testing::{
        MockChannelRepository, MockEventBus, MockMessageRepository, MockUserRepository,
    };
    use ruckchat_domain::{ConversationType, User};
    use ruckchat_id::UserId;
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;

    fn host(
        users: Arc<MockUserRepository>,
        events: Arc<MockEventBus>,
        messages: Arc<MockMessageRepository>,
    ) -> ServerHostApi {
        ServerHostApi::new(
            json!({"key": "value"}),
            users,
            Arc::new(MockChannelRepository::new()),
            messages,
            events,
        )
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_user_returns_seeded_user() {
        let users = Arc::new(MockUserRepository::new());
        let user = User::new("alice@example.com", "Alice", "hash").unwrap();
        users.create(&user).await.unwrap();

        let host = host(
            users,
            Arc::new(MockEventBus::new()),
            Arc::new(MockMessageRepository::new()),
        );
        let found = host.get_user(user.id).unwrap();
        assert_eq!(found.unwrap().email, "alice@example.com");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn send_message_persists_and_publishes() {
        let messages = Arc::new(MockMessageRepository::new());
        let events = Arc::new(MockEventBus::new());
        let host = host(
            Arc::new(MockUserRepository::new()),
            events.clone(),
            messages.clone(),
        );

        let request = SendMessageRequest {
            author_id: UserId::new(),
            conversation_id: Uuid::new_v4(),
            conversation_type: ConversationType::Channel,
            content: "plugin msg".into(),
            parent_id: None,
        };
        let message = host.send_message(request).unwrap();
        assert_eq!(message.content, "plugin msg");

        let found = messages.by_id(message.id).await.unwrap();
        assert!(found.is_some());

        assert!(events.events().iter().any(|e| matches!(
            e,
            ServerEvent::MessageCreated { message: m } if m.id == message.id
        )));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn emit_event_publishes_message_updated() {
        let events = Arc::new(MockEventBus::new());
        let host = host(
            Arc::new(MockUserRepository::new()),
            events.clone(),
            Arc::new(MockMessageRepository::new()),
        );

        let message = Message::new(
            Uuid::new_v4(),
            ConversationType::Channel,
            UserId::new(),
            "hello",
            None,
        )
        .unwrap();
        host.emit_event(PluginEvent::MessageUpdated {
            message: message.clone(),
        })
        .unwrap();

        assert!(events.events().iter().any(|e| matches!(
            e,
            ServerEvent::MessageUpdated { message: m } if m.id == message.id
        )));
    }
}
