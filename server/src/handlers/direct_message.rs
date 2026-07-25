//! Direct message route handlers.

use crate::{
    Error,
    handlers::{
        auth::AuthUser,
        dto::{ListResponse, MessagePageResponse, PostDmMessageRequest},
    },
    services::dto::{MarkReadRequest, MessagePageQuery, StartDmRequest},
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
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for listing direct message conversations.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDmParams {
    /// Organization that owns the conversations.
    pub organization_id: ruckchat_id::OrganizationId,
}

/// Lists direct message conversations for the caller in an organization.
pub async fn list_conversations(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<ListDmParams>,
) -> Result<Json<ListResponse<ruckchat_domain::DirectMessageConversation>>, Error> {
    let conversations = state
        .direct_messages
        .list_conversations_for_user(auth_user.id, params.organization_id)
        .await?;
    Ok(Json(ListResponse::new(conversations)))
}

/// Starts a direct message conversation.
pub async fn start(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<StartDmRequest>,
) -> Result<impl IntoResponse, Error> {
    let conversation = state
        .direct_messages
        .start_conversation(auth_user.id, request)
        .await?;
    Ok((StatusCode::CREATED, Json(conversation)))
}

/// Lists messages in a direct message conversation.
pub async fn list_messages(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<MessagePageQuery>,
) -> Result<Json<MessagePageResponse>, Error> {
    let page = state
        .messages
        .get_history(
            auth_user.id,
            conversation_id,
            ConversationType::DirectMessage,
            query,
        )
        .await?;
    let ids: Vec<MessageId> = page.messages.iter().map(|m| m.id).collect();
    let unread = state.read_state.unread_ids(auth_user.id, &ids).await?;
    Ok(Json(MessagePageResponse::from_page(
        page,
        auth_user.id,
        &unread,
    )))
}

/// Hides a direct message conversation from the caller's sidebar.
pub async fn hide(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(conversation_id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    state
        .direct_messages
        .hide_conversation(auth_user.id, conversation_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Posts a message to a direct message conversation.
pub async fn post_message(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<PostDmMessageRequest>,
) -> Result<impl IntoResponse, Error> {
    let service_request = crate::services::dto::PostMessageRequest {
        conversation_id,
        conversation_type: ConversationType::DirectMessage,
        parent_id: request.parent_id,
        content: request.content,
    };
    let message = state
        .messages
        .post_message(auth_user.id, service_request)
        .await?;
    Ok((StatusCode::CREATED, Json(message)))
}

/// Marks messages in a direct message conversation as read by the caller.
pub async fn mark_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<MarkReadRequest>,
) -> Result<StatusCode, Error> {
    state
        .read_state
        .mark_conversation_read(
            auth_user.id,
            conversation_id,
            ConversationType::DirectMessage,
            request.message_ids,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
