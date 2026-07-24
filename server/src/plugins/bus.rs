//! Event bus that forwards server events to WebSocket clients and plugins.

use crate::plugins::manager::PluginManager;
use crate::services::events::{EventBus, PresenceStatus};
use crate::services::web_push::WebPushService;
use crate::websocket::WebSocketEventBus;
use std::sync::Arc;

/// Event bus that multiplexes server events to WebSocket clients, loaded plugins,
/// and browser push subscriptions.
#[derive(Clone)]
pub struct CompositeEventBus {
    /// WebSocket event bus.
    websocket: WebSocketEventBus,
    /// Loaded plugin manager.
    plugins: Arc<PluginManager>,
    /// Optional Web Push notification service.
    web_push: Option<WebPushService>,
}

impl CompositeEventBus {
    /// Creates a composite bus from a WebSocket bus, a plugin manager, and an
    /// optional Web Push notification service.
    #[must_use]
    pub fn new(
        websocket: WebSocketEventBus,
        plugins: Arc<PluginManager>,
        web_push: Option<WebPushService>,
    ) -> Self {
        Self {
            websocket,
            plugins,
            web_push,
        }
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
        if let Some(web_push) = &self.web_push {
            web_push.publish_message_created(message).await.ok();
        }
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

    async fn publish_mention(
        &self,
        user_id: ruckchat_id::UserId,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        self.websocket.publish_mention(user_id, message).await
    }
}
