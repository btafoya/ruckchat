//! Embedded Hunspell spelling engine for RuckChat.
//!
//! The engine embeds LibreOffice `en-US.aff` and `en-US.dic` dictionaries at
//! compile time and exposes a thread-safe API for checking text and fetching
//! suggestions.

use spellbook::{Dictionary, ParseDictionaryError};
use std::sync::Arc;

/// A reported misspelling inside a larger text block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Misspelling {
    /// Byte offset where the misspelled word starts.
    pub offset: usize,
    /// Byte length of the misspelled word.
    pub length: usize,
    /// The misspelled word as it appeared in the text.
    pub word: String,
    /// Suggested corrections, capped to the caller's requested limit.
    pub suggestions: Vec<String>,
}

/// Thread-safe spelling engine backed by an embedded Hunspell dictionary.
#[derive(Debug, Clone)]
pub struct SpellingEngine {
    dict: Arc<Dictionary>,
    language: &'static str,
}

impl SpellingEngine {
    /// Creates the engine using the embedded en-US dictionaries.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedded dictionaries cannot be parsed. This
    /// should only fail if the bundled files are corrupted.
    pub fn embedded_en_us() -> Result<Self, ParseDictionaryError> {
        let aff = include_str!("../dictionaries/en-US.aff");
        let dic = include_str!("../dictionaries/en-US.dic");
        Self::new(aff, dic, "en-US")
    }

    /// Creates an engine from the supplied Hunspell `.aff` and `.dic` content.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary content cannot be parsed.
    pub fn new(aff: &str, dic: &str, language: &'static str) -> Result<Self, ParseDictionaryError> {
        let dict = Dictionary::new(aff, dic)?;
        Ok(Self {
            dict: Arc::new(dict),
            language,
        })
    }

    /// Returns the configured language tag.
    #[must_use]
    pub fn language(&self) -> &'static str {
        self.language
    }

    /// Returns suggestions for a single word, capped at `max`.
    #[must_use]
    pub fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        let mut all = Vec::new();
        self.dict.suggest(word, &mut all);
        all.into_iter().take(max).collect()
    }

    /// Scans `text` for misspelled words and returns each one with suggestions.
    ///
    /// `max_suggestions` controls how many suggestions are returned per
    /// misspelling. Set it to `0` to skip suggestion generation.
    #[must_use]
    pub fn check(&self, text: &str, max_suggestions: usize) -> Vec<Misspelling> {
        let mut misspellings = Vec::new();
        for (offset, length, word) in words(text) {
            if word.is_empty() || self.dict.check(word) {
                continue;
            }
            let suggestions = if max_suggestions == 0 {
                Vec::new()
            } else {
                self.suggest(word, max_suggestions)
            };
            misspellings.push(Misspelling {
                offset,
                length,
                word: word.to_string(),
                suggestions,
            });
        }
        misspellings
    }
}

/// Iterates over alphabetic word tokens in `text`, yielding byte offset,
/// byte length, and the lowercased token.
fn words(text: &str) -> impl Iterator<Item = (usize, usize, &str)> + use<'_> {
    let mut start: Option<usize> = None;
    let mut chars = text.char_indices().peekable();
    std::iter::from_fn(move || {
        for (offset, ch) in chars.by_ref() {
            if ch.is_alphabetic() {
                start = Some(start.unwrap_or(offset));
            } else if let Some(s) = start.take() {
                return Some((s, offset - s, &text[s..offset]));
            }
        }
        start.take().map(|s| (s, text.len() - s, &text[s..]))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_misspellings() {
        let engine = SpellingEngine::embedded_en_us().expect("embedded dictionaries should parse");
        let result = engine.check("The quikc brown foxx jumps.", 3);
        let words: Vec<String> = result.iter().map(|m| m.word.clone()).collect();
        assert!(words.contains(&"quikc".to_string()));
        assert!(words.contains(&"foxx".to_string()));
    }

    #[test]
    fn recognizes_correct_words() {
        let engine = SpellingEngine::embedded_en_us().expect("embedded dictionaries should parse");
        assert!(engine.check("Hello world", 3).is_empty());
    }

    #[test]
    fn suggest_returns_corrections() {
        let engine = SpellingEngine::embedded_en_us().expect("embedded dictionaries should parse");
        let suggestions = engine.suggest("foxx", 5);
        assert!(
            suggestions.iter().any(|s| s == "fox" || s == "foxy"),
            "expected 'fox' or 'foxy' in suggestions, got {:?}",
            suggestions
        );
    }

    #[test]
    fn language_returns_en_us() {
        let engine = SpellingEngine::embedded_en_us().expect("embedded dictionaries should parse");
        assert_eq!(engine.language(), "en-US");
    }
}
