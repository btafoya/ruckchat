//! User aggregate.

use ruckchat_common::{
    Error, Result,
    time::OffsetDateTime,
    validate_email,
    validation::{DISPLAY_NAME_MAX_LEN, DISPLAY_NAME_MIN_LEN, validate_display_name},
};
use ruckchat_id::UserId;
use serde::{Deserialize, Serialize};

/// A human account in RuckChat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    /// Internal user identifier.
    pub id: UserId,
    /// Globally unique email address.
    pub email: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Argon2 password hash. The domain layer stores but does not generate this.
    pub password_hash: String,
    /// Optional URL to an avatar image.
    pub avatar_url: Option<String>,
    /// Timestamp when the user was deactivated, if applicable.
    #[serde(with = "time::serde::rfc3339::option")]
    pub deactivated_at: Option<OffsetDateTime>,
    /// Timestamp when the user was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Timestamp of the last profile update.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl User {
    /// Creates a new user after validating email format and display name length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the email or display name is invalid.
    pub fn new(
        email: impl Into<String>,
        display_name: impl Into<String>,
        password_hash: impl Into<String>,
    ) -> Result<Self> {
        let email = email.into();
        let display_name = display_name.into();
        let password_hash = password_hash.into();

        if !validate_email(&email) {
            return Err(Error::validation(format!("invalid email: {email}")));
        }
        if !validate_display_name(&display_name) {
            return Err(Error::validation(format!(
                "display name must be {DISPLAY_NAME_MIN_LEN}-{DISPLAY_NAME_MAX_LEN} characters"
            )));
        }
        if password_hash.is_empty() {
            return Err(Error::validation("password hash must not be empty"));
        }

        let now = OffsetDateTime::now_utc();
        Ok(Self {
            id: UserId::new(),
            email,
            display_name,
            password_hash,
            avatar_url: None,
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates the display name after validating length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the new name is invalid.
    pub fn set_display_name(&mut self, display_name: impl Into<String>) -> Result<()> {
        let display_name = display_name.into();
        if !validate_display_name(&display_name) {
            return Err(Error::validation(format!(
                "display name must be {DISPLAY_NAME_MIN_LEN}-{DISPLAY_NAME_MAX_LEN} characters"
            )));
        }
        self.display_name = display_name;
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }

    /// Updates the avatar URL.
    pub fn set_avatar_url(&mut self, avatar_url: Option<impl Into<String>>) {
        self.avatar_url = avatar_url.map(Into::into);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Deactivates the user account.
    pub fn deactivate(&mut self) {
        if self.deactivated_at.is_none() {
            self.deactivated_at = Some(OffsetDateTime::now_utc());
            self.updated_at = OffsetDateTime::now_utc();
        }
    }

    /// Reactivates a previously deactivated user account.
    pub fn reactivate(&mut self) {
        if self.deactivated_at.is_some() {
            self.deactivated_at = None;
            self.updated_at = OffsetDateTime::now_utc();
        }
    }

    /// Returns true if the user has been deactivated.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.deactivated_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_user() {
        let user = User::new("alice@example.com", "Alice", "hash").expect("valid user");
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.display_name, "Alice");
        assert_eq!(user.password_hash, "hash");
        assert!(user.avatar_url.is_none());
    }

    #[test]
    fn invalid_email_rejected() {
        assert!(User::new("not-an-email", "Alice", "hash").is_err());
    }

    #[test]
    fn empty_password_hash_rejected() {
        assert!(User::new("alice@example.com", "Alice", "").is_err());
    }

    #[test]
    fn bad_display_name_rejected() {
        assert!(User::new("alice@example.com", "", "hash").is_err());
        assert!(User::new("alice@example.com", "x".repeat(101), "hash").is_err());
    }

    #[test]
    fn update_display_name() {
        let mut user = User::new("alice@example.com", "Alice", "hash").expect("valid user");
        let before = user.updated_at;
        user.set_display_name("Alice Updated").expect("update name");
        assert_eq!(user.display_name, "Alice Updated");
        assert!(user.updated_at >= before);
    }

    #[test]
    fn update_avatar_url() {
        let mut user = User::new("alice@example.com", "Alice", "hash").expect("valid user");
        user.set_avatar_url(Some("https://example.com/avatar.png"));
        assert_eq!(
            user.avatar_url,
            Some("https://example.com/avatar.png".into())
        );
    }

    #[test]
    fn deactivate_and_reactivate() {
        let mut user = User::new("alice@example.com", "Alice", "hash").expect("valid user");
        assert!(user.is_active());
        user.deactivate();
        assert!(!user.is_active());
        assert!(user.deactivated_at.is_some());
        user.reactivate();
        assert!(user.is_active());
        assert!(user.deactivated_at.is_none());
    }
}
