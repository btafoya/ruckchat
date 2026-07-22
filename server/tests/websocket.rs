//! WebSocket real-time event integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use futures_util::{SinkExt, StreamExt};
use ruckchat_server::{handlers::router, state::AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, Utf8Bytes},
};

/// Starts an in-process server and returns a REST test client plus the
/// WebSocket base URL. Both share the same [`AppState`] so REST mutations emit
/// events to WebSocket connections.
async fn start_server(pool: PgPool) -> (TestClient, String) {
    ruckchat_migrations::migrator()
        .run(&pool)
        .await
        .expect("migrations apply");
    let state = AppState::from_pool(pool, false);
    let app = router().with_state(state);
    let client = TestClient::from_router(app.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("ws://127.0.0.1:{}", addr.port());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (client, base_url)
}

async fn register_and_login(client: &TestClient) -> (String, String, String) {
    let email = test_email("websocket");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Owner",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": uuid::Uuid::new_v4().to_string()
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let register_body = body_json(response).await;
    let org_id = register_body["organization"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({
                "email": email,
                "password": "correct horse battery staple"
            })),
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let token = body["token"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/channels"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let channel_id = body["items"][0]["id"].as_str().unwrap().to_string();

    (token, org_id, channel_id)
}

async fn connect_ws(
    base_url: &str,
    token: &str,
) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    let mut request = format!("{base_url}/websocket")
        .into_client_request()
        .expect("valid websocket url");
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {token}").parse().unwrap());
    let (ws, response) = tokio_tungstenite::connect_async(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    ws
}

async fn recv_event(ws: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>) -> Value {
    let timeout = tokio::time::timeout(Duration::from_secs(2), ws.next());
    let msg = timeout
        .await
        .expect("websocket event received within timeout")
        .expect("websocket stream not closed")
        .expect("websocket message is valid");
    match msg {
        Message::Text(text) => serde_json::from_str::<Value>(&text).expect("event is valid json"),
        other => panic!("expected text message, got {other:?}"),
    }
}

async fn recv_event_of_type(
    ws: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    event_type: &str,
) -> Value {
    loop {
        let event = recv_event(ws).await;
        if event["type"] == event_type {
            return event;
        }
    }
}

async fn send_json(ws: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, value: &Value) {
    let text = serde_json::to_string(value).unwrap();
    ws.send(Message::Text(Utf8Bytes::from(text))).await.unwrap();
}

#[sqlx::test]
async fn websocket_connect_receives_established_event(pool: PgPool) {
    let (client, base_url) = start_server(pool).await;
    let (token, _, _) = register_and_login(&client).await;

    let mut ws = connect_ws(&base_url, &token).await;
    let event = recv_event(&mut ws).await;
    assert_eq!(event["type"], "connection.established");
}

#[sqlx::test]
async fn websocket_receives_message_created_event(pool: PgPool) {
    let (client, base_url) = start_server(pool).await;
    let (token, _org_id, channel_id) = register_and_login(&client).await;

    let mut ws = connect_ws(&base_url, &token).await;
    let welcome = recv_event(&mut ws).await;
    assert_eq!(welcome["type"], "connection.established");
    let _presence = recv_event_of_type(&mut ws, "presence.updated").await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "hello websocket" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let event = recv_event_of_type(&mut ws, "message.created").await;
    assert_eq!(event["payload"]["message"]["content"], "hello websocket");
}

#[sqlx::test]
async fn websocket_receives_reaction_events(pool: PgPool) {
    let (client, base_url) = start_server(pool).await;
    let (token, _org_id, channel_id) = register_and_login(&client).await;

    let mut ws = connect_ws(&base_url, &token).await;
    let _welcome = recv_event(&mut ws).await;
    let _presence = recv_event_of_type(&mut ws, "presence.updated").await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "react to me" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let message_id = body_json(response).await["id"]
        .as_str()
        .unwrap()
        .to_string();
    let _message_event = recv_event_of_type(&mut ws, "message.created").await;

    let response = client
        .auth_request(
            "POST",
            &format!("/messages/{message_id}/reactions"),
            &token,
            Some(json!({ "emoji": "👍" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let event = recv_event_of_type(&mut ws, "reaction.added").await;
    assert_eq!(event["payload"]["reaction"]["emoji"], "👍");

    let response = client
        .auth_request(
            "DELETE",
            &format!("/messages/{message_id}/reactions/%F0%9F%91%8D"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::NO_CONTENT);

    let event = recv_event_of_type(&mut ws, "reaction.removed").await;
    assert_eq!(event["payload"]["emoji"], "👍");
}

#[sqlx::test]
async fn websocket_receives_typing_event(pool: PgPool) {
    let (client, base_url) = start_server(pool).await;
    let (token, _org_id, channel_id) = register_and_login(&client).await;

    let mut ws = connect_ws(&base_url, &token).await;
    let _welcome = recv_event(&mut ws).await;
    let _presence = recv_event_of_type(&mut ws, "presence.updated").await;

    send_json(
        &mut ws,
        &json!({
            "type": "typing",
            "conversation_id": channel_id,
            "conversation_type": "channel"
        }),
    )
    .await;

    let event = recv_event_of_type(&mut ws, "typing.updated").await;
    assert_eq!(event["payload"]["conversation_id"], channel_id);
}
