//! Transactional email sending for RuckChat via Postmark.
//!
//! [`EmailClient`] wraps the `postmark` crate's reqwest-backed client. Message
//! bodies are composed inline as HTML/text, not via Postmark's server-side
//! template feature, so wording changes ship through a normal code deploy
//! rather than a Postmark-dashboard edit.

use postmark::api::Body;
use postmark::api::email::SendEmailRequest;
use postmark::reqwest::{PostmarkClient, PostmarkClientError};
use postmark::{Query, QueryError};
use serde::{Deserialize, Serialize};

/// Postmark credentials and sender address, loaded from `ruckchat.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// Postmark server API token.
    pub server_token: String,
    /// From address used for all outgoing email.
    pub from_address: String,
}

/// Errors returned by [`EmailClient`] send operations.
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// The Postmark API request failed.
    #[error("postmark send failed: {0}")]
    Send(#[from] QueryError<PostmarkClientError>),
}

/// Sends transactional email through Postmark.
#[derive(Clone)]
pub struct EmailClient {
    client: PostmarkClient,
    from_address: String,
}

impl EmailClient {
    /// Creates a client from Postmark configuration.
    #[must_use]
    pub fn new(config: &EmailConfig) -> Self {
        Self {
            client: PostmarkClient::builder()
                .server_token(config.server_token.clone())
                .build(),
            from_address: config.from_address.clone(),
        }
    }

    /// Sends a migrated user their real, usable temporary password.
    ///
    /// # Errors
    ///
    /// Returns [`EmailError`] when the Postmark API request fails.
    pub async fn send_migration_credentials(
        &self,
        to: &str,
        temp_password: &str,
    ) -> Result<(), EmailError> {
        self.send(
            to,
            "Your RuckChat account",
            &format!(
                "<p>Your RocketChat account has been migrated to RuckChat.</p>\
                 <p>Log in with your email address and this temporary password, \
                 then change it from your profile settings:</p>\
                 <p><strong>{temp_password}</strong></p>"
            ),
            &format!(
                "Your RocketChat account has been migrated to RuckChat.\n\n\
                 Log in with your email address and this temporary password, \
                 then change it from your profile settings:\n\n{temp_password}\n"
            ),
        )
        .await
    }

    /// Sends a server-admin-triggered password reset.
    ///
    /// # Errors
    ///
    /// Returns [`EmailError`] when the Postmark API request fails.
    pub async fn send_password_reset(
        &self,
        to: &str,
        temp_password: &str,
    ) -> Result<(), EmailError> {
        self.send(
            to,
            "Your RuckChat password was reset",
            &format!(
                "<p>An administrator reset your RuckChat password.</p>\
                 <p>Your new temporary password is:</p>\
                 <p><strong>{temp_password}</strong></p>\
                 <p>Log in and change it from your profile settings.</p>"
            ),
            &format!(
                "An administrator reset your RuckChat password.\n\n\
                 Your new temporary password is:\n\n{temp_password}\n\n\
                 Log in and change it from your profile settings.\n"
            ),
        )
        .await
    }

    async fn send(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<(), EmailError> {
        let request = SendEmailRequest::builder()
            .from(self.from_address.clone())
            .to(to)
            .subject(subject)
            .body(Body::html_and_text(html.to_string(), text.to_string()))
            .build();
        request.execute(&self.client).await?;
        Ok(())
    }
}
