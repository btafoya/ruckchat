//! Session aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{SessionId, UserId};
use serde::{Deserialize, Serialize};

/// An authenticated browser or client session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Internal session identifier.
    pub id: SessionId,
    /// User this session belongs to.
    pub user_id: UserId,
    /// Hashed session token. The domain layer stores but does not generate this.
    pub token_hash: String,
    /// Expiration timestamp.
    pub expires_at: OffsetDateTime,
    /// Timestamp when the session was created.
    pub created_at: OffsetDateTime,
    /// Optional IP address of the client.
    pub ip_address: Option<String>,
    /// Optional user agent string.
    pub user_agent: Option<String>,
}

impl Session {
    /// Creates a new session after validating that the token hash is non-empty
    /// and the expiration is in the future.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when input is invalid.
    pub fn new(
        user_id: UserId,
        token_hash: impl Into<String>,
        expires_at: OffsetDateTime,
        ip_address: Option<impl Into<String>>,
        user_agent: Option<impl Into<String>>,
    ) -> Result<Self> {
        let token_hash = token_hash.into();
        if token_hash.is_empty() {
            return Err(Error::validation("token hash must not be empty"));
        }
        if expires_at <= OffsetDateTime::now_utc() {
            return Err(Error::validation(
                "session expiration must be in the future",
            ));
        }

        Ok(Self {
            id: SessionId::new(),
            user_id,
            token_hash,
            expires_at,
            created_at: OffsetDateTime::now_utc(),
            ip_address: ip_address.map(Into::into),
            user_agent: user_agent.map(Into::into),
        })
    }

    /// Returns true if the session has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() >= self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_session() {
        let user_id = UserId::new();
        let expires = OffsetDateTime::now_utc() + time::Duration::hours(1);
        let session = Session::new(user_id, "hash", expires, None::<&str>, None::<&str>)
            .expect("valid session");
        assert_eq!(session.user_id, user_id);
        assert!(!session.is_expired());
    }

    #[test]
    fn empty_token_hash_rejected() {
        let expires = OffsetDateTime::now_utc() + time::Duration::hours(1);
        assert!(Session::new(UserId::new(), "", expires, None::<&str>, None::<&str>).is_err());
    }

    #[test]
    fn past_expiration_rejected() {
        let expires = OffsetDateTime::now_utc() - time::Duration::hours(1);
        assert!(Session::new(UserId::new(), "hash", expires, None::<&str>, None::<&str>).is_err());
    }

    #[test]
    fn expired_session_detected() {
        let user_id = UserId::new();
        let expires = OffsetDateTime::now_utc() - time::Duration::seconds(1);
        let session = Session {
            id: SessionId::new(),
            user_id,
            token_hash: "hash".into(),
            expires_at: expires,
            created_at: expires - time::Duration::hours(1),
            ip_address: None,
            user_agent: None,
        };
        assert!(session.is_expired());
    }
}
