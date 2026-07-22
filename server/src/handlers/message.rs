//! Message route handlers.

use crate::{
    Error,
    handlers::{
        auth::AuthUser,
        dto::{ListResponse, PostChannelMessageRequest},
    },
    services::dto::{EditMessageRequest, Pagination},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_domain::ConversationType;
use ruckchat_id::MessageId;
use uuid::Uuid;

/// Lists message history for a channel.
pub async fn list_history(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<ListResponse<ruckchat_domain::Message>>, Error> {
    let messages = state
        .messages
        .get_history(
            auth_user.id,
            channel_id,
            ConversationType::Channel,
            pagination.normalized(),
        )
        .await?;
    Ok(Json(ListResponse::new(messages)))
}

/// Posts a message to a channel.
pub async fn post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(request): Json<PostChannelMessageRequest>,
) -> Result<impl IntoResponse, Error> {
    let service_request = crate::services::dto::PostMessageRequest {
        conversation_id: channel_id,
        conversation_type: ConversationType::Channel,
        parent_id: request.parent_id,
        content: request.content,
    };
    let message = state
        .messages
        .post_message(auth_user.id, service_request)
        .await?;
    Ok((StatusCode::CREATED, Json(message)))
}

/// Edits an existing message.
pub async fn edit(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<ruckchat_domain::Message>, Error> {
    let message = state
        .messages
        .edit_message(auth_user.id, MessageId::from_uuid(message_id), request)
        .await?;
    Ok(Json(message))
}

/// Soft-deletes a message.
pub async fn delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    state
        .messages
        .delete_message(auth_user.id, MessageId::from_uuid(message_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Lists thread replies for a message.
pub async fn list_replies(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<ListResponse<ruckchat_domain::Message>>, Error> {
    let replies = state
        .messages
        .get_thread_replies(
            auth_user.id,
            MessageId::from_uuid(message_id),
            pagination.normalized(),
        )
        .await?;
    Ok(Json(ListResponse::new(replies)))
}
