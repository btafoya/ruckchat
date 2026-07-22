//! Reaction route handlers.

use crate::{Error, handlers::auth::AuthUser, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_id::MessageId;
use serde::Deserialize;
use uuid::Uuid;

/// Request to add a reaction to a message.
#[derive(Debug, Clone, Deserialize)]
pub struct AddReactionRequest {
    /// Emoji character or shortcode.
    pub emoji: String,
}

/// Adds a reaction to a message.
pub async fn add(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
    Json(request): Json<AddReactionRequest>,
) -> Result<impl IntoResponse, Error> {
    let reaction = state
        .reactions
        .add_reaction(
            auth_user.id,
            MessageId::from_uuid(message_id),
            request.emoji,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(reaction)))
}

/// Removes the caller's reaction from a message.
pub async fn remove(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((message_id, emoji)): Path<(Uuid, String)>,
) -> Result<StatusCode, Error> {
    state
        .reactions
        .remove_reaction(auth_user.id, MessageId::from_uuid(message_id), &emoji)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
