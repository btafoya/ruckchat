//! Authentication service.

use crate::{
    Error, Result,
    services::dto::{LoginRequest, LoginResponse, RegisterRequest},
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::RngCore},
};
use rand::rngs::OsRng;
use ruckchat_common::time::OffsetDateTime;
use ruckchat_domain::{
    Channel, ChannelMembership, Organization, OrganizationMembership, OrganizationSettings, Role,
    Session, User,
};
use ruckchat_id::{ChannelId, OrganizationId, UserId};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Default session lifetime.
const SESSION_DURATION_DAYS: i64 = 30;

/// Minimum password length enforced at registration.
const MIN_PASSWORD_LENGTH: usize = 10;

/// Dependencies required by [`AuthService`].
#[derive(Clone)]
pub struct AuthServiceDeps {
    /// User repository.
    pub users: Arc<dyn ruckchat_domain::UserRepository + Send + Sync>,
    /// Session repository.
    pub sessions: Arc<dyn ruckchat_domain::SessionRepository + Send + Sync>,
    /// Organization repository.
    pub organizations: Arc<dyn ruckchat_domain::OrganizationRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync>,
    /// Organization settings repository.
    pub settings: Arc<dyn ruckchat_domain::OrganizationSettingsRepository + Send + Sync>,
    /// Channel repository.
    pub channels: Arc<dyn ruckchat_domain::ChannelRepository + Send + Sync>,
    /// Channel membership repository.
    pub channel_memberships: Arc<dyn ruckchat_domain::ChannelMembershipRepository + Send + Sync>,
}

/// Registers users, verifies passwords, and manages sessions.
#[derive(Clone)]
pub struct AuthService {
    deps: AuthServiceDeps,
}

impl AuthService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: AuthServiceDeps) -> Self {
        Self { deps }
    }

    /// Registers a new user with an initial organization and `#general` channel.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Validation`] for invalid input, [`ruckchat_common::Error::Conflict`] if
    /// the email or organization slug already exists, and [`ruckchat_common::Error::Internal`]
    /// for hashing failures.
    pub async fn register(
        &self,
        request: RegisterRequest,
    ) -> Result<(ruckchat_domain::User, ruckchat_domain::Organization)> {
        if request.password.len() < MIN_PASSWORD_LENGTH {
            return Err(Error::Domain(ruckchat_common::Error::validation(format!(
                "password must be at least {MIN_PASSWORD_LENGTH} characters"
            ))));
        }

        if let Some(existing) = self.deps.users.by_email(&request.email).await? {
            return Err(Error::Domain(ruckchat_common::Error::Conflict(format!(
                "user already exists: {}",
                existing.email
            ))));
        }

        if self
            .deps
            .organizations
            .by_slug(&request.organization_slug)
            .await?
            .is_some()
        {
            return Err(Error::Domain(ruckchat_common::Error::Conflict(
                "organization slug already exists".into(),
            )));
        }

        let password_hash = hash_password(&request.password)?;
        let mut user = User::new(request.email, request.display_name, password_hash)?;

        // The first registered user becomes a server administrator.
        let is_first_user = self.deps.users.count().await? == 0;
        if is_first_user {
            user.set_server_admin(true);
        }
        let organization = Organization::new(
            request.organization_name,
            request.organization_slug,
            user.id,
        )?;

        self.deps.users.create(&user).await.map_err(|_| {
            Error::Domain(ruckchat_common::Error::Conflict(
                "email already in use".into(),
            ))
        })?;

        self.deps.organizations.create(&organization).await?;

        let membership = OrganizationMembership::new(user.id, organization.id, Role::Owner)?;
        self.deps.memberships.create(&membership).await?;

        let settings = OrganizationSettings::new(organization.id);
        self.deps.settings.create(&settings).await?;

        let general = Channel::new(organization.id, "general", user.id, false)?;
        self.deps.channels.create(&general).await?;

        let general_membership = ChannelMembership::new(user.id, general.id)?;
        self.deps
            .channel_memberships
            .create(&general_membership)
            .await?;

        Ok((user, organization))
    }

    /// Verifies credentials and creates a session.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Unauthorized`] for bad credentials and [`ruckchat_common::Error::Internal`]
    /// for hashing failures.
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        let user = self
            .deps
            .users
            .by_email(&request.email)
            .await?
            .ok_or_else(|| {
                Error::Domain(ruckchat_common::Error::Unauthorized(
                    "invalid credentials".into(),
                ))
            })?;

        verify_password(&request.password, &user.password_hash)?;

        let (token, token_hash) = generate_session_token();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS);
        let session = Session::new(user.id, token_hash, expires_at, None::<&str>, None::<&str>)?;

        self.deps.sessions.create(&session).await?;

        Ok(LoginResponse {
            token,
            user_id: user.id,
        })
    }

    /// Invalidates a session by token hash.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Unauthorized`] when the session does not exist or is expired.
    pub async fn logout(&self, token: &str) -> Result<()> {
        let token_hash = hash_token(token);
        let session = self
            .deps
            .sessions
            .by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| {
                Error::Domain(ruckchat_common::Error::Unauthorized(
                    "session not found".into(),
                ))
            })?;

        if session.is_expired() {
            return Err(Error::Domain(ruckchat_common::Error::Unauthorized(
                "session expired".into(),
            )));
        }

        self.deps.sessions.delete_by_token_hash(&token_hash).await?;
        Ok(())
    }

    /// Loads a session by token, returning the user id if valid.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Unauthorized`] when the session is missing or expired.
    pub async fn authenticate(&self, token: &str) -> Result<UserId> {
        let session = self.session_by_token(token).await?;
        Ok(session.user_id)
    }

    /// Loads a session by token.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Unauthorized`] when the session is missing or expired.
    pub async fn session_by_token(&self, token: &str) -> Result<Session> {
        let token_hash = hash_token(token);
        let session = self
            .deps
            .sessions
            .by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| {
                Error::Domain(ruckchat_common::Error::Unauthorized(
                    "session not found".into(),
                ))
            })?;

        if session.is_expired() {
            return Err(Error::Domain(ruckchat_common::Error::Unauthorized(
                "session expired".into(),
            )));
        }

        Ok(session)
    }

    /// Removes expired sessions.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Internal`] for database failures.
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let count = self.deps.sessions.delete_expired().await?;
        Ok(count)
    }

    /// Creates an impersonation session for a target user and returns the token.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Internal`] for session creation failures.
    pub async fn create_impersonation_session(
        &self,
        target_user_id: UserId,
        impersonated_by: UserId,
    ) -> Result<String> {
        let (token, token_hash) = generate_session_token();
        let expires_at =
            ruckchat_common::time::OffsetDateTime::now_utc() + time::Duration::hours(1);
        let mut session = Session::new(
            target_user_id,
            token_hash,
            expires_at,
            None::<&str>,
            None::<&str>,
        )?;
        session.impersonated_by = Some(impersonated_by);
        self.deps.sessions.create(&session).await?;
        Ok(token)
    }

    /// Ends an impersonation session by token.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Unauthorized`] when the session is missing
    /// or not an impersonation session.
    pub async fn end_impersonation_session(&self, token: &str) -> Result<()> {
        let token_hash = hash_token(token);
        let session = self
            .deps
            .sessions
            .by_token_hash(&token_hash)
            .await?
            .ok_or_else(|| {
                Error::Domain(ruckchat_common::Error::Unauthorized(
                    "session not found".into(),
                ))
            })?;
        if session.impersonated_by.is_none() {
            return Err(Error::Domain(ruckchat_common::Error::Unauthorized(
                "not an impersonation session".into(),
            )));
        }
        self.deps.sessions.delete_by_token_hash(&token_hash).await?;
        Ok(())
    }
}

pub(crate) fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| Error::PasswordHash)?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<()> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| Error::PasswordHash)?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            Error::Domain(ruckchat_common::Error::Unauthorized(
                "invalid credentials".into(),
            ))
        })
}

fn generate_session_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = hex::encode(bytes);
    (token.clone(), hash_token(&token))
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

// Dummy uses of imports that are referenced by integration tests but not yet
// used in this module's public API. These keep the compiler happy until the
// handlers phase consumes them directly.
const _: fn() = || {
    let _: ChannelId = ChannelId::new();
    let _: OrganizationId = OrganizationId::new();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dto::{LoginRequest, RegisterRequest};
    use crate::testing::{
        MockChannelMembershipRepository, MockChannelRepository,
        MockOrganizationMembershipRepository, MockOrganizationRepository,
        MockOrganizationSettingsRepository, MockSessionRepository, MockUserRepository,
    };

    fn auth_service() -> AuthService {
        AuthService::new(AuthServiceDeps {
            users: Arc::new(MockUserRepository::new()),
            sessions: Arc::new(MockSessionRepository::new()),
            organizations: Arc::new(MockOrganizationRepository::new()),
            memberships: Arc::new(MockOrganizationMembershipRepository::new()),
            settings: Arc::new(MockOrganizationSettingsRepository::new()),
            channels: Arc::new(MockChannelRepository::new()),
            channel_memberships: Arc::new(MockChannelMembershipRepository::new()),
        })
    }

    #[tokio::test]
    async fn register_creates_user_and_organization() {
        let service = auth_service();
        let req = RegisterRequest {
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            password: "correct horse battery staple".into(),
            organization_name: "Acme".into(),
            organization_slug: "acme".into(),
        };

        let (user, org) = service.register(req).await.expect("register");
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(org.slug, "acme");
    }

    #[tokio::test]
    async fn register_rejects_short_password() {
        let service = auth_service();
        let req = RegisterRequest {
            email: "bob@example.com".into(),
            display_name: "Bob".into(),
            password: "short".into(),
            organization_name: "Acme".into(),
            organization_slug: "acme2".into(),
        };

        let err = service.register(req).await.unwrap_err();
        assert!(matches!(
            err,
            Error::Domain(ruckchat_common::Error::Validation { .. })
        ));
    }

    #[tokio::test]
    async fn register_rejects_duplicate_email() {
        let service = auth_service();
        let req = RegisterRequest {
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            password: "correct horse battery staple".into(),
            organization_name: "Acme".into(),
            organization_slug: "acme".into(),
        };
        service.register(req.clone()).await.expect("first register");
        let err = service.register(req).await.unwrap_err();
        assert!(matches!(
            err,
            Error::Domain(ruckchat_common::Error::Conflict { .. })
        ));
    }

    #[tokio::test]
    async fn login_succeeds_with_valid_credentials() {
        let service = auth_service();
        let req = RegisterRequest {
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            password: "correct horse battery staple".into(),
            organization_name: "Acme".into(),
            organization_slug: "acme".into(),
        };
        let (user, _) = service.register(req).await.expect("register");

        let resp = service
            .login(LoginRequest {
                email: "alice@example.com".into(),
                password: "correct horse battery staple".into(),
            })
            .await
            .expect("login");
        assert_eq!(resp.user_id, user.id);
        assert!(!resp.token.is_empty());
    }

    #[tokio::test]
    async fn login_fails_with_wrong_password() {
        let service = auth_service();
        let req = RegisterRequest {
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            password: "correct horse battery staple".into(),
            organization_name: "Acme".into(),
            organization_slug: "acme".into(),
        };
        service.register(req).await.expect("register");

        let err = service
            .login(LoginRequest {
                email: "alice@example.com".into(),
                password: "wrong password".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            Error::Domain(ruckchat_common::Error::Unauthorized { .. })
        ));
    }
}
