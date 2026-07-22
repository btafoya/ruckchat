//! Shared primitives for RuckChat: errors, validation, and time helpers.

use once_cell::sync::Lazy;
use regex::Regex;

pub mod error;
pub mod time;
pub mod validation;

pub use error::{Error, Result};

/// Validates an email address using a permissive but RFC-aware regex.
pub fn validate_email(email: &str) -> bool {
    static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        )
        .expect("email regex is valid")
    });
    EMAIL_RE.is_match(email)
}

/// Checks whether a value is a valid URL-safe organization or channel slug.
pub fn validate_slug(slug: &str) -> bool {
    static SLUG_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("slug regex is valid"));
    SLUG_RE.is_match(slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_emails() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("first+tag@sub.domain.org"));
    }

    #[test]
    fn invalid_emails() {
        assert!(!validate_email("plainstring"));
        assert!(!validate_email("@nodomain.com"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn valid_slugs() {
        assert!(validate_slug("acme-corp"));
        assert!(validate_slug("team-42"));
    }

    #[test]
    fn invalid_slugs() {
        assert!(!validate_slug("UpperCase"));
        assert!(!validate_slug("under_score"));
        assert!(!validate_slug("-leading"));
        assert!(!validate_slug("trailing-"));
        assert!(!validate_slug("double--hyphen"));
    }
}
