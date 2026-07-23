//! Web Push notification service.

use async_trait::async_trait;
use ruckchat_common::{Error, Result};
use ruckchat_domain::{
    ChannelMembershipRepository, DirectMessageConversationRepository, Message, UserRepository,
    WebPushSubscription, WebPushSubscriptionRepository,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, UserId};
use std::sync::Arc;
use uuid::Uuid;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, URL_SAFE_NO_PAD, VapidSignatureBuilder,
    WebPushClient, WebPushMessageBuilder,
};

/// Dependencies required by [`WebPushService`].
#[derive(Clone)]
pub struct WebPushServiceDeps {
    /// Web Push subscription repository.
    pub subscriptions: Arc<dyn WebPushSubscriptionRepository + Send + Sync>,
    /// DM conversation repository (used to find DM recipients).
    pub conversations: Arc<dyn DirectMessageConversationRepository + Send + Sync>,
    /// Channel membership repository (used to restrict mention notifications).
    pub channel_memberships: Arc<dyn ChannelMembershipRepository + Send + Sync>,
    /// User repository (used to resolve author display names).
    pub users: Arc<dyn UserRepository + Send + Sync>,
}

/// Runtime configuration used to sign push messages.
#[derive(Clone)]
pub struct WebPushServiceConfig {
    /// VAPID subject claim, typically a `mailto:` URI.
    pub subject: String,
    /// VAPID private key in unpadded URL-safe base64.
    pub private_key: String,
    /// VAPID public key exposed to browser clients.
    pub public_key: String,
}

impl WebPushServiceConfig {
    /// Builds a configuration from the application config, returning `None` when
    /// Web Push is disabled or keys are missing.
    pub fn from_config(config: &ruckchat_config::WebPushConfig) -> Option<Self> {
        if !config.enabled {
            return None;
        }
        let subject = config.subject.clone()?;
        let private_key = config.vapid_private_key.clone()?;
        let public_key = config.vapid_public_key.clone()?;
        if private_key.is_empty() || public_key.is_empty() {
            return None;
        }
        Some(Self {
            subject,
            private_key,
            public_key,
        })
    }
}

/// Sends browser Web Push notifications for messages and other events.
#[derive(Clone)]
pub struct WebPushService {
    deps: WebPushServiceDeps,
    config: WebPushServiceConfig,
    client: Arc<IsahcWebPushClient>,
}

impl WebPushService {
    /// Creates the service from dependencies and configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Internal`] when the push client cannot be initialized.
    pub fn new(deps: WebPushServiceDeps, config: WebPushServiceConfig) -> Result<Self> {
        let client = Arc::new(
            IsahcWebPushClient::new()
                .map_err(|err| Error::Internal(format!("web push client: {err}")))?,
        );
        Ok(Self {
            deps,
            config,
            client,
        })
    }

    /// Returns the VAPID public key that browsers use to subscribe.
    #[must_use]
    pub fn public_key(&self) -> &str {
        &self.config.public_key
    }

    /// Stores or updates a push subscription for a user.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the subscription fields are empty or
    /// [`Error::Internal`] on persistence failure.
    pub async fn subscribe(
        &self,
        user_id: UserId,
        endpoint: String,
        p256dh: String,
        auth: String,
    ) -> Result<()> {
        let subscription = WebPushSubscription::new(user_id, endpoint, p256dh, auth)?;
        self.deps.subscriptions.upsert(&subscription).await
    }

    /// Removes a push subscription for a user.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Internal`] on persistence failure.
    pub async fn unsubscribe(&self, user_id: UserId, endpoint: String) -> Result<()> {
        self.deps
            .subscriptions
            .delete_by_endpoint(user_id, &endpoint)
            .await
    }

    /// Sends a push notification for a newly created message.
    pub async fn notify_for_message(&self, message: &Message) {
        let recipients = match self.recipients_for_message(message).await {
            Some(recipients) => recipients,
            None => return,
        };

        let (title, body) = match self.notification_text(message).await {
            Some(text) => text,
            None => return,
        };

        for user_id in recipients {
            if let Err(err) = self.notify(user_id, &title, &body).await {
                tracing::warn!(%err, %user_id, "failed to send web push notification");
            }
        }
    }

    /// Sends a push notification to a single user.
    ///
    /// Stale endpoints (`EndpointNotFound` / `EndpointNotValid`) cause the
    /// subscription to be deleted.
    pub async fn notify(&self, user_id: UserId, title: &str, body: &str) -> Result<()> {
        let subscriptions = self.deps.subscriptions.list_by_user(user_id).await?;
        if subscriptions.is_empty() {
            return Ok(());
        }

        let payload = serde_json::json!({
            "title": title,
            "body": body,
        })
        .to_string();
        let payload_bytes = payload.into_bytes();

        for subscription in subscriptions {
            if let Err(err) = self
                .send_to_subscription(&subscription, &payload_bytes)
                .await
            {
                if should_remove_subscription(&err) {
                    tracing::info!(
                        %user_id,
                        endpoint = %subscription.endpoint,
                        "removing stale web push subscription"
                    );
                    if let Err(delete_err) = self
                        .deps
                        .subscriptions
                        .delete_by_endpoint(user_id, &subscription.endpoint)
                        .await
                    {
                        tracing::warn!(
                            %delete_err,
                            %user_id,
                            endpoint = %subscription.endpoint,
                            "failed to delete stale web push subscription"
                        );
                    }
                } else {
                    tracing::warn!(
                        %user_id,
                        endpoint = %subscription.endpoint,
                        %err,
                        "web push send failed"
                    );
                }
            }
        }

        Ok(())
    }

    async fn send_to_subscription(
        &self,
        subscription: &WebPushSubscription,
        payload: &[u8],
    ) -> std::result::Result<(), web_push::WebPushError> {
        let info = SubscriptionInfo::new(
            &subscription.endpoint,
            &subscription.p256dh,
            &subscription.auth,
        );

        let mut sig_builder =
            VapidSignatureBuilder::from_base64(&self.config.private_key, URL_SAFE_NO_PAD, &info)?;
        sig_builder.add_claim("sub", self.config.subject.clone());
        let signature = sig_builder.build()?;

        let mut builder = WebPushMessageBuilder::new(&info);
        builder.set_payload(ContentEncoding::Aes128Gcm, payload);
        builder.set_vapid_signature(signature);

        let message = builder.build()?;
        self.client.send(message).await
    }

    async fn recipients_for_message(&self, message: &Message) -> Option<Vec<UserId>> {
        match message.conversation_type {
            ruckchat_domain::ConversationType::DirectMessage => {
                let conversation_id =
                    DirectMessageConversationId::from_uuid(message.conversation_id);
                let conversation = self
                    .deps
                    .conversations
                    .by_id(conversation_id)
                    .await
                    .ok()
                    .flatten()?;
                Some(
                    conversation
                        .member_ids
                        .into_iter()
                        .filter(|id| *id != message.author_id)
                        .collect(),
                )
            }
            ruckchat_domain::ConversationType::Channel => {
                let channel_id = ChannelId::from_uuid(message.conversation_id);
                let memberships = self
                    .deps
                    .channel_memberships
                    .list_by_channel(channel_id)
                    .await
                    .ok()?;
                let member_ids: std::collections::HashSet<UserId> = memberships
                    .into_iter()
                    .map(|m| m.user_id)
                    .filter(|id| *id != message.author_id)
                    .collect();

                let mentions = parse_mention_ids(&message.content);
                let mut recipients = Vec::new();
                for user_id in mentions {
                    if member_ids.contains(&user_id) {
                        recipients.push(user_id);
                    }
                }
                Some(recipients)
            }
        }
    }

    async fn notification_text(&self, message: &Message) -> Option<(String, String)> {
        let author = self
            .deps
            .users
            .by_id(message.author_id)
            .await
            .ok()
            .flatten()?;
        let title = match message.conversation_type {
            ruckchat_domain::ConversationType::DirectMessage => "New direct message".into(),
            ruckchat_domain::ConversationType::Channel => "New mention".into(),
        };
        let snippet: String = message
            .content
            .chars()
            .take(120)
            .collect::<String>()
            .trim_end()
            .into();
        let body = format!("{}: {}", author.display_name, snippet);
        Some((title, body))
    }
}

fn parse_mention_ids(content: &str) -> Vec<UserId> {
    let mut ids = Vec::new();
    for token in content.split_whitespace() {
        if let Some(stripped) = token.strip_prefix('@')
            && let Ok(uuid) = Uuid::parse_str(stripped)
        {
            ids.push(UserId::from_uuid(uuid));
        }
    }
    ids
}

fn should_remove_subscription(err: &web_push::WebPushError) -> bool {
    matches!(
        err,
        web_push::WebPushError::EndpointNotFound
            | web_push::WebPushError::EndpointNotValid
            | web_push::WebPushError::InvalidUri
    )
}

#[async_trait]
impl crate::services::events::EventBus for WebPushService {
    async fn publish_message_created(&self, message: &Message) -> ruckchat_common::Result<()> {
        self.notify_for_message(message).await;
        Ok(())
    }

    async fn publish_message_updated(&self, _message: &Message) -> ruckchat_common::Result<()> {
        Ok(())
    }

    async fn publish_message_deleted(&self, _message: &Message) -> ruckchat_common::Result<()> {
        Ok(())
    }

    async fn publish_reaction_added(
        &self,
        _reaction: &ruckchat_domain::Reaction,
    ) -> ruckchat_common::Result<()> {
        Ok(())
    }

    async fn publish_reaction_removed(
        &self,
        _message_id: ruckchat_id::MessageId,
        _user_id: UserId,
        _emoji: &str,
    ) -> ruckchat_common::Result<()> {
        Ok(())
    }

    async fn publish_typing(
        &self,
        _user_id: UserId,
        _conversation_id: Uuid,
        _conversation_type: ruckchat_domain::ConversationType,
    ) -> ruckchat_common::Result<()> {
        Ok(())
    }

    async fn publish_presence(
        &self,
        _user_id: UserId,
        _status: crate::services::events::PresenceStatus,
    ) -> ruckchat_common::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        MockChannelMembershipRepository, MockDirectMessageConversationRepository,
        MockUserRepository, MockWebPushSubscriptionRepository,
    };
    use ruckchat_domain::{ConversationType, DirectMessageConversation};
    use ruckchat_id::{ChannelId, OrganizationId};

    fn service() -> WebPushService {
        let deps = WebPushServiceDeps {
            subscriptions: Arc::new(MockWebPushSubscriptionRepository::new()),
            conversations: Arc::new(MockDirectMessageConversationRepository::new()),
            channel_memberships: Arc::new(MockChannelMembershipRepository::new()),
            users: Arc::new(MockUserRepository::new()),
        };
        WebPushService::new(
            deps,
            WebPushServiceConfig {
                subject: "mailto:test@example.com".into(),
                private_key: "dGVzdC1rZXk".into(),
                public_key: "cHVibGljLWtleQ".into(),
            },
        )
        .expect("valid service")
    }

    #[test]
    fn parse_mention_ids_extracts_uuids() {
        let id = UserId::new();
        let content = format!("hello @{} and @not-a-uuid", id);
        let parsed = parse_mention_ids(&content);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], id);
    }

    #[tokio::test]
    async fn dm_recipients_exclude_author() {
        let svc = service();
        let org_id = OrganizationId::new();
        let author = UserId::new();
        let other = UserId::new();
        let conversation = DirectMessageConversation::new(org_id, vec![author, other]).unwrap();
        svc.deps.conversations.create(&conversation).await.unwrap();

        let message = Message::new(
            conversation.id.as_uuid(),
            ConversationType::DirectMessage,
            author,
            "hello",
            None,
        )
        .unwrap();

        let recipients = svc.recipients_for_message(&message).await.unwrap();
        assert_eq!(recipients, vec![other]);
    }

    #[tokio::test]
    async fn channel_mentions_require_membership() {
        let svc = service();
        let channel_id = ChannelId::new();
        let author = UserId::new();
        let member = UserId::new();
        let outsider = UserId::new();

        svc.deps
            .channel_memberships
            .create(&ruckchat_domain::ChannelMembership {
                user_id: member,
                channel_id,
                joined_at: time::OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();

        let message = Message::new(
            channel_id.as_uuid(),
            ConversationType::Channel,
            author,
            format!("hey @{} @{}", member, outsider),
            None,
        )
        .unwrap();

        let recipients = svc.recipients_for_message(&message).await.unwrap();
        assert_eq!(recipients, vec![member]);
    }
}
