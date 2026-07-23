//! Event bus that forwards server events to WebSocket clients and plugins.

use crate::plugins::manager::PluginManager;
use crate::services::events::{EventBus, PresenceStatus};
use crate::websocket::WebSocketEventBus;
use std::sync::Arc;

/// Event bus that multiplexes server events to both WebSocket clients and
/// loaded plugins.
#[derive(Clone)]
pub struct CompositeEventBus {
    /// WebSocket event bus.
    websocket: WebSocketEventBus,
    /// Loaded plugin manager.
    plugins: Arc<PluginManager>,
}

impl CompositeEventBus {
    /// Creates a composite bus from a WebSocket bus and a plugin manager.
    #[must_use]
    pub fn new(websocket: WebSocketEventBus, plugins: Arc<PluginManager>) -> Self {
        Self { websocket, plugins }
    }
}

#[async_trait::async_trait]
impl EventBus for CompositeEventBus {
    async fn publish_message_created(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        self.websocket.publish_message_created(message).await?;
        self.plugins
            .dispatch_event(ruckchat_plugin_sdk::PluginEvent::MessageReceived {
                message: message.clone(),
            });
        Ok(())
    }

    async fn publish_message_updated(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        self.websocket.publish_message_updated(message).await?;
        self.plugins
            .dispatch_event(ruckchat_plugin_sdk::PluginEvent::MessageUpdated {
                message: message.clone(),
            });
        Ok(())
    }

    async fn publish_message_deleted(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        self.websocket.publish_message_deleted(message).await?;
        self.plugins
            .dispatch_event(ruckchat_plugin_sdk::PluginEvent::MessageDeleted {
                message: message.clone(),
            });
        Ok(())
    }

    async fn publish_reaction_added(
        &self,
        reaction: &ruckchat_domain::Reaction,
    ) -> ruckchat_common::Result<()> {
        self.websocket.publish_reaction_added(reaction).await
    }

    async fn publish_reaction_removed(
        &self,
        message_id: ruckchat_id::MessageId,
        user_id: ruckchat_id::UserId,
        emoji: &str,
    ) -> ruckchat_common::Result<()> {
        self.websocket
            .publish_reaction_removed(message_id, user_id, emoji)
            .await
    }

    async fn publish_typing(
        &self,
        user_id: ruckchat_id::UserId,
        conversation_id: uuid::Uuid,
        conversation_type: ruckchat_domain::ConversationType,
    ) -> ruckchat_common::Result<()> {
        self.websocket
            .publish_typing(user_id, conversation_id, conversation_type)
            .await
    }

    async fn publish_presence(
        &self,
        user_id: ruckchat_id::UserId,
        status: PresenceStatus,
    ) -> ruckchat_common::Result<()> {
        self.websocket.publish_presence(user_id, status).await
    }
}
