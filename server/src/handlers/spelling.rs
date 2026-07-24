//! Spell-checker REST handlers.

use crate::{handlers::AuthUser, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
};
use ruckchat_spelling::Misspelling;
use serde::{Deserialize, Serialize};

/// Request to check a block of text.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckRequest {
    /// Text to check.
    pub text: String,
    /// Maximum suggestions per misspelling.
    #[serde(default)]
    pub max_suggestions: Option<usize>,
}

/// A misspelling inside the checked text.
#[derive(Debug, Clone, Serialize)]
pub struct CheckMisspelling {
    /// Byte offset where the misspelled word starts.
    pub offset: usize,
    /// Byte length of the misspelled word.
    pub length: usize,
    /// The misspelled word.
    pub word: String,
    /// Suggested corrections.
    pub suggestions: Vec<String>,
}

impl From<Misspelling> for CheckMisspelling {
    fn from(value: Misspelling) -> Self {
        Self {
            offset: value.offset,
            length: value.length,
            word: value.word,
            suggestions: value.suggestions,
        }
    }
}

/// Response from a spell-check request.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResponse {
    /// Detected misspellings.
    pub misspellings: Vec<CheckMisspelling>,
}

/// Request to get suggestions for a single word.
#[derive(Debug, Clone, Deserialize)]
pub struct SuggestRequest {
    /// Word to suggest corrections for.
    pub word: String,
    /// Maximum suggestions to return.
    #[serde(default)]
    pub max: Option<usize>,
}

/// Response from a suggestion request.
#[derive(Debug, Clone, Serialize)]
pub struct SuggestResponse {
    /// Suggested corrections.
    pub suggestions: Vec<String>,
}

/// Response listing available spell-checker languages.
#[derive(Debug, Clone, Serialize)]
pub struct LanguagesResponse {
    /// Supported language tags.
    pub languages: Vec<String>,
}

/// Query parameters for the languages endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct LanguagesQuery {
    /// Optional language filter; ignored by the current implementation.
    #[serde(default)]
    pub filter: Option<String>,
}
/// `POST /api/v1/spelling/check` — checks text and returns misspellings.
pub async fn check(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, crate::Error> {
    let enabled = state
        .server_settings
        .load()
        .await
        .map(|s| s.spelling_enabled)
        .unwrap_or(false);
    if !enabled {
        return Ok(Json(CheckResponse {
            misspellings: Vec::new(),
        }));
    }
    let Some(spelling) = state.spelling.as_ref() else {
        return Ok(Json(CheckResponse {
            misspellings: Vec::new(),
        }));
    };
    let misspellings = spelling
        .check(auth_user.id, request.text, request.max_suggestions)
        .await?;
    Ok(Json(CheckResponse {
        misspellings: misspellings.into_iter().map(Into::into).collect(),
    }))
}

/// `POST /api/v1/spelling/suggest` — returns suggestions for a word.
pub async fn suggest(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<SuggestRequest>,
) -> Result<Json<SuggestResponse>, crate::Error> {
    let enabled = state
        .server_settings
        .load()
        .await
        .map(|s| s.spelling_enabled)
        .unwrap_or(false);
    if !enabled {
        return Ok(Json(SuggestResponse {
            suggestions: Vec::new(),
        }));
    }
    let Some(spelling) = state.spelling.as_ref() else {
        return Ok(Json(SuggestResponse {
            suggestions: Vec::new(),
        }));
    };
    let suggestions = spelling
        .suggest(auth_user.id, request.word, request.max)
        .await?;
    Ok(Json(SuggestResponse { suggestions }))
}

/// `GET /api/v1/spelling/languages` — returns supported language tags.
pub async fn languages(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Query(_query): Query<LanguagesQuery>,
) -> Result<Json<LanguagesResponse>, crate::Error> {
    let languages = if let Some(spelling) = state.spelling.as_ref() {
        vec![spelling.language().to_string()]
    } else {
        Vec::new()
    };
    Ok(Json(LanguagesResponse { languages }))
}
