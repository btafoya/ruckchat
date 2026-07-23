//! Plugin command route handlers.

use crate::{Error, handlers::auth::AuthUser, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_domain::ConversationType;
use ruckchat_plugin_sdk::{CommandResponse, PluginCommand};
use serde::Deserialize;
use uuid::Uuid;

/// Request body for invoking a plugin command.
#[derive(Debug, Clone, Deserialize)]
pub struct InvokeCommandRequest {
    /// Conversation the command was invoked in.
    pub conversation_id: Uuid,
    /// Conversation kind.
    pub conversation_type: ConversationType,
    /// Positional arguments.
    #[serde(default)]
    pub args: Vec<String>,
}

/// Invokes a plugin command.
///
/// The caller must be authenticated. The plugin receives the caller's user id
/// and the requested conversation; it is responsible for any authorization
/// checks beyond being authenticated.
pub async fn invoke_command(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((plugin, command)): Path<(String, String)>,
    Json(request): Json<InvokeCommandRequest>,
) -> Result<impl IntoResponse, Error> {
    let command = PluginCommand {
        plugin,
        command,
        args: request.args,
        conversation_id: request.conversation_id,
        conversation_type: request.conversation_type,
        user_id: auth_user.id,
    };

    let response = state
        .plugin_manager
        .dispatch_command(command)
        .map_err(|err| {
            ruckchat_common::Error::NotFound(match err {
                crate::plugins::manager::PluginManagerError::NotFound(name) => {
                    format!("plugin not found: {name}")
                }
                crate::plugins::manager::PluginManagerError::Io(err) => err.to_string(),
            })
        })?;

    let status = match response {
        CommandResponse::Message { .. } | CommandResponse::Ephemeral { .. } => StatusCode::OK,
        CommandResponse::Error { .. } => StatusCode::BAD_REQUEST,
    };

    Ok((status, Json(response)))
}
