//! Server-side spell-checking service.
//!
//! Wraps the embedded [`ruckchat_spelling::SpellingEngine`] with per-user
//! token-bucket rate limiting and input validation.

use ruckchat_common::Error;
use ruckchat_id::UserId;
use ruckchat_spelling::{Misspelling, SpellingEngine};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::Mutex;

/// Maximum text length accepted by the spell-checker.
const MAX_TEXT_LENGTH: usize = 10_000;
/// Maximum length of a single word accepted for suggestions.
const MAX_WORD_LENGTH: usize = 100;
/// Maximum number of suggestions returned per misspelling or word.
const MAX_SUGGESTIONS: usize = 10;

/// Burst limit and refill rate for the per-second token bucket.
const PER_SECOND_BURST: u32 = 10;
const PER_SECOND_RATE: f64 = 10.0;

/// Burst limit and refill rate for the per-minute token bucket.
const PER_MINUTE_BURST: u32 = 100;
const PER_MINUTE_RATE: f64 = 100.0 / 60.0;

/// State tracked per user for rate limiting.
#[derive(Debug, Clone, Copy)]
struct Bucket {
    tokens: f64,
    last_update: Instant,
}

impl Bucket {
    fn new(burst: u32) -> Self {
        Self {
            tokens: f64::from(burst),
            last_update: Instant::now(),
        }
    }

    fn consume(&mut self, burst: u32, rate_per_second: f64, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * rate_per_second).min(f64::from(burst));
        self.last_update = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Per-user rate-limit state.
#[derive(Debug, Clone, Copy)]
struct UserLimit {
    per_second: Bucket,
    per_minute: Bucket,
}

impl UserLimit {
    fn new() -> Self {
        Self {
            per_second: Bucket::new(PER_SECOND_BURST),
            per_minute: Bucket::new(PER_MINUTE_BURST),
        }
    }

    fn check(&mut self, now: Instant) -> bool {
        self.per_second
            .consume(PER_SECOND_BURST, PER_SECOND_RATE, now)
            && self
                .per_minute
                .consume(PER_MINUTE_BURST, PER_MINUTE_RATE, now)
    }
}

/// Dependencies required by [`SpellingService`].
pub struct SpellingServiceDeps {
    /// Embedded spelling engine.
    pub engine: SpellingEngine,
}

/// Spell-checking service with per-user rate limiting.
#[derive(Debug, Clone)]
pub struct SpellingService {
    engine: SpellingEngine,
    limits: Arc<Mutex<HashMap<UserId, UserLimit>>>,
}

impl SpellingService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: SpellingServiceDeps) -> Self {
        Self {
            engine: deps.engine,
            limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns the configured language tag.
    #[must_use]
    pub fn language(&self) -> &'static str {
        self.engine.language()
    }

    /// Checks `text` for misspellings, returning each one with suggestions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the input is too long or otherwise
    /// invalid, and [`Error::TooManyRequests`] when the caller exceeds the
    /// per-user rate limit.
    pub async fn check(
        &self,
        caller_id: UserId,
        text: String,
        max_suggestions: Option<usize>,
    ) -> Result<Vec<Misspelling>, Error> {
        self.enforce_rate_limit(caller_id).await?;
        validate_text(&text)?;
        let max = max_suggestions
            .map(|m| m.min(MAX_SUGGESTIONS))
            .unwrap_or(MAX_SUGGESTIONS);
        Ok(self.engine.check(&text, max))
    }

    /// Returns suggestions for a single word.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the word is too long or invalid, and
    /// [`Error::TooManyRequests`] when the caller exceeds the per-user rate
    /// limit.
    pub async fn suggest(
        &self,
        caller_id: UserId,
        word: String,
        max: Option<usize>,
    ) -> Result<Vec<String>, Error> {
        self.enforce_rate_limit(caller_id).await?;
        validate_word(&word)?;
        let max = max
            .map(|m| m.min(MAX_SUGGESTIONS))
            .unwrap_or(MAX_SUGGESTIONS);
        Ok(self.engine.suggest(&word, max))
    }

    /// Applies the per-user token-bucket rate limit.
    async fn enforce_rate_limit(&self, caller_id: UserId) -> Result<(), Error> {
        let now = Instant::now();
        let mut limits = self.limits.lock().await;
        let limit = limits.entry(caller_id).or_insert_with(UserLimit::new);
        if limit.check(now) {
            Ok(())
        } else {
            Err(Error::TooManyRequests(
                "spell-checker rate limit exceeded".into(),
            ))
        }
    }
}

fn validate_text(text: &str) -> Result<(), Error> {
    if text.len() > MAX_TEXT_LENGTH {
        return Err(Error::validation(format!(
            "text exceeds maximum length of {MAX_TEXT_LENGTH} bytes"
        )));
    }
    Ok(())
}

fn validate_word(word: &str) -> Result<(), Error> {
    if word.len() > MAX_WORD_LENGTH {
        return Err(Error::validation(format!(
            "word exceeds maximum length of {MAX_WORD_LENGTH} characters"
        )));
    }
    if word.is_empty() {
        return Err(Error::validation("word is required"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> SpellingService {
        SpellingService::new(SpellingServiceDeps {
            engine: SpellingEngine::embedded_en_us().expect("embedded dictionaries should parse"),
        })
    }

    #[tokio::test]
    async fn check_finds_misspelling() {
        let svc = service();
        let result = svc
            .check(UserId::new(), "foxx".to_string(), Some(3))
            .await
            .expect("allowed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].word, "foxx");
    }

    #[tokio::test]
    async fn suggest_returns_corrections() {
        let svc = service();
        let result = svc
            .suggest(UserId::new(), "foxx".to_string(), Some(5))
            .await
            .expect("allowed");
        assert!(
            result.iter().any(|s| s == "fox" || s == "foxy"),
            "expected 'fox' or 'foxy' in {:?}",
            result
        );
    }

    #[tokio::test]
    async fn rejects_oversized_text() {
        let svc = service();
        let text = "a".repeat(MAX_TEXT_LENGTH + 1);
        let err = svc.check(UserId::new(), text, None).await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn rejects_oversized_word() {
        let svc = service();
        let word = "a".repeat(MAX_WORD_LENGTH + 1);
        let err = svc.suggest(UserId::new(), word, None).await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn rate_limits_after_burst() {
        let svc = service();
        let caller = UserId::new();
        for _ in 0..PER_SECOND_BURST {
            svc.check(caller, "hello world".to_string(), Some(1))
                .await
                .expect("within burst");
        }
        let err = svc
            .check(caller, "hello world".to_string(), Some(1))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::TooManyRequests { .. }));
    }
}
