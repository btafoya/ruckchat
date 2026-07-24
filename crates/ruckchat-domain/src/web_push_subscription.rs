//! Web Push subscription domain model.

use ruckchat_id::UserId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// A browser's Web Push subscription belonging to a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebPushSubscription {
    /// Internal subscription identifier.
    pub id: Uuid,
    /// User who owns the subscription.
    pub user_id: UserId,
    /// Push service endpoint URL.
    pub endpoint: String,
    /// P-256 ECDH public key, base64url encoded.
    pub p256dh: String,
    /// Authentication secret, base64url encoded.
    pub auth: String,
    /// Timestamp when the subscription was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Timestamp of the last update.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl WebPushSubscription {
    /// Creates a new subscription for the given user and browser keys.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Validation`] when any required field is
    /// empty.
    pub fn new(
        user_id: UserId,
        endpoint: impl Into<String>,
        p256dh: impl Into<String>,
        auth: impl Into<String>,
    ) -> ruckchat_common::Result<Self> {
        let endpoint = endpoint.into();
        let p256dh = p256dh.into();
        let auth = auth.into();

        if endpoint.is_empty() {
            return Err(ruckchat_common::Error::validation(
                "push endpoint must not be empty",
            ));
        }
        if p256dh.is_empty() {
            return Err(ruckchat_common::Error::validation(
                "push p256dh key must not be empty",
            ));
        }
        if auth.is_empty() {
            return Err(ruckchat_common::Error::validation(
                "push auth secret must not be empty",
            ));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            endpoint,
            p256dh,
            auth,
            created_at: now,
            updated_at: now,
        })
    }
}
