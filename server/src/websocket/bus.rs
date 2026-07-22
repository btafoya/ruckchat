//! WebSocket event bus implementation.
//!
//! Resolves event recipients through repositories and dispatches envelopes via
//! the in-memory connection manager.

use crate::{
    services::events::{EventBus, EventEnvelope, PresenceStatus, ServerEvent},
    websocket::manager::ConnectionManager,
};
use ruckchat_domain::{
    ChannelMembershipRepository, ChannelRepository, ConversationType, MessageRepository,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, MessageId, UserId};
use std::sync::Arc;
use uuid::Uuid;

/// Dependencies for the WebSocket event bus.
#[derive(Clone)]
pub struct WebSocketEventBusDeps {
    /// Connection registry.
    pub manager: ConnectionManager,
    /// Message repository.
    pub messages: Arc<dyn MessageRepository + Send + Sync>,
    /// Channel repository.
    pub channels: Arc<dyn ChannelRepository + Send + Sync>,
    /// Channel membership repository.
    pub channel_memberships: Arc<dyn ChannelMembershipRepository + Send + Sync>,
    /// DM conversation repository.
    pub conversations: Arc<dyn ruckchat_domain::DirectMessageConversationRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync>,
}

/// Broadcasts events to connected WebSocket clients.
#[derive(Clone)]
pub struct WebSocketEventBus {
    deps: WebSocketEventBusDeps,
}

impl WebSocketEventBus {
    /// Creates the event bus from its dependencies.
    #[must_use]
    pub fn new(deps: WebSocketEventBusDeps) -> Self {
        Self { deps }
    }
}

#[async_trait::async_trait]
impl EventBus for WebSocketEventBus {
    async fn publish_message_created(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        let envelope = EventEnvelope::new(ServerEvent::MessageCreated {
            message: message.clone(),
        });
        self.broadcast_to_conversation(
            message.conversation_id,
            message.conversation_type,
            envelope,
        )
        .await;
        Ok(())
    }

    async fn publish_message_updated(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        let envelope = EventEnvelope::new(ServerEvent::MessageUpdated {
            message: message.clone(),
        });
        self.broadcast_to_conversation(
            message.conversation_id,
            message.conversation_type,
            envelope,
        )
        .await;
        Ok(())
    }

    async fn publish_message_deleted(
        &self,
        message: &ruckchat_domain::Message,
    ) -> ruckchat_common::Result<()> {
        let envelope = EventEnvelope::new(ServerEvent::MessageDeleted {
            message: message.clone(),
        });
        self.broadcast_to_conversation(
            message.conversation_id,
            message.conversation_type,
            envelope,
        )
        .await;
        Ok(())
    }

    async fn publish_reaction_added(
        &self,
        reaction: &ruckchat_domain::Reaction,
    ) -> ruckchat_common::Result<()> {
        let message = match self.deps.messages.by_id(reaction.message_id).await {
            Ok(Some(message)) => message,
            _ => return Ok(()),
        };
        let envelope = EventEnvelope::new(ServerEvent::ReactionAdded {
            reaction: reaction.clone(),
        });
        self.broadcast_to_conversation(
            message.conversation_id,
            message.conversation_type,
            envelope,
        )
        .await;
        Ok(())
    }

    async fn publish_reaction_removed(
        &self,
        message_id: MessageId,
        user_id: UserId,
        emoji: &str,
    ) -> ruckchat_common::Result<()> {
        let message = match self.deps.messages.by_id(message_id).await {
            Ok(Some(message)) => message,
            _ => return Ok(()),
        };
        let envelope = EventEnvelope::new(ServerEvent::ReactionRemoved {
            message_id,
            user_id,
            emoji: emoji.into(),
        });
        self.broadcast_to_conversation(
            message.conversation_id,
            message.conversation_type,
            envelope,
        )
        .await;
        Ok(())
    }

    async fn publish_typing(
        &self,
        user_id: UserId,
        conversation_id: Uuid,
        conversation_type: ConversationType,
    ) -> ruckchat_common::Result<()> {
        let envelope = EventEnvelope::new(ServerEvent::Typing {
            user_id,
            conversation_id,
            conversation_type,
        });
        self.broadcast_typing_to_conversation(conversation_id, conversation_type, envelope)
            .await;
        Ok(())
    }

    async fn publish_presence(
        &self,
        user_id: UserId,
        status: PresenceStatus,
    ) -> ruckchat_common::Result<()> {
        let envelope = EventEnvelope::new(ServerEvent::Presence { user_id, status });
        let memberships = match self.deps.memberships.list_by_user(user_id).await {
            Ok(memberships) => memberships,
            Err(err) => {
                tracing::warn!(%err, "failed to load user memberships for presence broadcast");
                return Ok(());
            }
        };
        for membership in memberships {
            self.deps
                .manager
                .broadcast_to_organization(membership.organization_id, envelope.clone())
                .await;
        }
        Ok(())
    }
}

impl WebSocketEventBus {
    async fn broadcast_to_conversation(
        &self,
        conversation_id: Uuid,
        conversation_type: ConversationType,
        envelope: EventEnvelope,
    ) {
        match conversation_type {
            ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(conversation_id);
                let channel = match self.deps.channels.by_id(channel_id).await {
                    Ok(Some(channel)) => channel,
                    Ok(None) => return,
                    Err(err) => {
                        tracing::warn!(%err, "failed to load channel for broadcast");
                        return;
                    }
                };
                if channel.is_private {
                    let members = match self
                        .deps
                        .channel_memberships
                        .list_by_channel(channel_id)
                        .await
                    {
                        Ok(members) => members,
                        Err(err) => {
                            tracing::warn!(%err, "failed to load channel members for broadcast");
                            return;
                        }
                    };
                    let user_ids: Vec<UserId> = members.into_iter().map(|m| m.user_id).collect();
                    self.deps
                        .manager
                        .broadcast_to_users(&user_ids, envelope)
                        .await;
                } else {
                    self.deps
                        .manager
                        .broadcast_to_organization(channel.organization_id, envelope)
                        .await;
                }
            }
            ConversationType::DirectMessage => {
                let conversation_id = DirectMessageConversationId::from_uuid(conversation_id);
                let conversation = match self.deps.conversations.by_id(conversation_id).await {
                    Ok(Some(conversation)) => conversation,
                    Ok(None) => return,
                    Err(err) => {
                        tracing::warn!(%err, "failed to load dm conversation for broadcast");
                        return;
                    }
                };
                self.deps
                    .manager
                    .broadcast_to_users(&conversation.member_ids, envelope)
                    .await;
            }
        }
    }

    async fn broadcast_typing_to_conversation(
        &self,
        conversation_id: Uuid,
        conversation_type: ConversationType,
        envelope: EventEnvelope,
    ) {
        match conversation_type {
            ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(conversation_id);
                let members = match self
                    .deps
                    .channel_memberships
                    .list_by_channel(channel_id)
                    .await
                {
                    Ok(members) => members,
                    Err(err) => {
                        tracing::warn!(%err, "failed to load channel members for typing broadcast");
                        return;
                    }
                };
                let user_ids: Vec<UserId> = members.into_iter().map(|m| m.user_id).collect();
                self.deps
                    .manager
                    .broadcast_to_users(&user_ids, envelope)
                    .await;
            }
            ConversationType::DirectMessage => {
                let conversation_id = DirectMessageConversationId::from_uuid(conversation_id);
                let conversation = match self.deps.conversations.by_id(conversation_id).await {
                    Ok(Some(conversation)) => conversation,
                    Ok(None) => return,
                    Err(err) => {
                        tracing::warn!(%err, "failed to load dm conversation for typing broadcast");
                        return;
                    }
                };
                self.deps
                    .manager
                    .broadcast_to_users(&conversation.member_ids, envelope)
                    .await;
            }
        }
    }
}
