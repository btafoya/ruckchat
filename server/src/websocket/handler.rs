//! WebSocket upgrade handler and per-connection loop.

use crate::{
    handlers::auth::AuthUser,
    services::events::{
        ClientMessage, ErrorEvent, EventBus, EventEnvelope, PresenceStatus, ServerEvent,
    },
    state::AppState,
    websocket::manager::ConnectionId,
};
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, ws::WebSocketUpgrade},
    response::Response,
};
use ruckchat_id::UserId;

/// Handles WebSocket upgrade requests.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, auth_user.id, state))
}

async fn handle_socket(mut socket: WebSocket, user_id: UserId, state: AppState) {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let manager = state.websocket_manager.clone();
    let events = state.events.clone();
    let connection_id = manager.register(user_id, event_tx).await;

    // Auto-subscribe to every organization the user belongs to.
    if let Ok(memberships) = state.users.list_memberships_for_user(user_id).await {
        for membership in memberships {
            manager
                .subscribe_organization(connection_id, membership.organization_id)
                .await;
        }
    }

    // Notify the client that the connection is ready.
    let welcome = EventEnvelope::new(ServerEvent::ConnectionEstablished { user_id });
    if send_text(&mut socket, &welcome).await.is_err() {
        manager.unregister(connection_id).await;
        return;
    }

    // Notify other clients that this user is online.
    let _ = events
        .publish_presence(user_id, PresenceStatus::Online)
        .await;

    loop {
        tokio::select! {
            Some(envelope) = event_rx.recv() => {
                if send_text(&mut socket, &envelope).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(err) = handle_client_message(
                            &state, connection_id, user_id, &text,
                        ).await {
                            let error = EventEnvelope::new(ServerEvent::Error {
                                error: ErrorEvent {
                                    code: "invalid_message".into(),
                                    message: err,
                                },
                            });
                            if send_text(&mut socket, &error).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {
                        // Ignore pings, pongs, and binary frames; native
                        // WebSocket heartbeat is handled by axum/tungstenite.
                    }
                    Some(Err(err)) => {
                        tracing::warn!(%err, "websocket receive error");
                        break;
                    }
                }
            }
        }
    }

    manager.unregister(connection_id).await;
    if manager.connection_count_for_user(user_id).await == 0 {
        let _ = events
            .publish_presence(user_id, PresenceStatus::Offline)
            .await;
    }
}

async fn send_text(socket: &mut WebSocket, envelope: &EventEnvelope) -> Result<(), ()> {
    let text = serde_json::to_string(envelope).map_err(|err| {
        tracing::warn!(%err, "failed to serialize event envelope");
    })?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|err| {
            tracing::warn!(%err, "websocket send error");
        })
}

async fn handle_client_message(
    state: &AppState,
    connection_id: ConnectionId,
    user_id: UserId,
    text: &str,
) -> Result<(), String> {
    let message: ClientMessage =
        serde_json::from_str(text).map_err(|err| format!("invalid client message: {err}"))?;

    match message {
        ClientMessage::SubscribeOrganization { organization_id } => {
            state
                .websocket_manager
                .subscribe_organization(connection_id, organization_id)
                .await;
        }
        ClientMessage::UnsubscribeOrganization { organization_id } => {
            state
                .websocket_manager
                .unsubscribe_organization(connection_id, organization_id)
                .await;
        }
        ClientMessage::Typing {
            conversation_id,
            conversation_type,
        } => {
            let _ = state
                .events
                .publish_typing(user_id, conversation_id, conversation_type)
                .await;
        }
        ClientMessage::Ping => {
            // Native WebSocket pong frames are handled automatically; an
            // application-level ping is accepted and ignored.
        }
    }

    Ok(())
}
