//! MCP Streamable HTTP endpoint integration tests.

mod common;

use axum::{
    Router,
    body::Body,
    http::{
        Request, StatusCode,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST},
    },
    response::Response,
};
use bytes::Bytes;
use common::{TestClient, assert_status, body_json, test_email};
use futures_util::StreamExt;
use ruckchat_server::{handlers::router, state::AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::time::Duration;
use tower::ServiceExt;

async fn setup_app(pool: PgPool, mcp_enabled: bool) -> (Router, TestClient) {
    ruckchat_migrations::migrator()
        .run(&pool)
        .await
        .expect("migrations apply");
    let state = AppState::from_pool(pool, false, mcp_enabled, true, "./plugins".into());
    let app = router().with_state(state);
    let client = TestClient::from_router(app.clone());
    (app, client)
}

async fn setup_user(client: &TestClient) -> (String, String, String) {
    let email = test_email("mcp");
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

fn mcp_request(token: &str, session_id: Option<&str>, body: Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp/v1/sse")
        .header(HOST, "localhost")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/json, text/event-stream")
        .header(CONTENT_TYPE, "application/json");
    if let Some(id) = session_id {
        builder = builder.header("mcp-session-id", id);
    }
    builder
        .body(Body::from(body.to_string()))
        .expect("valid request")
}

async fn sse_first_message(response: Response) -> Value {
    let mut stream = response.into_body().into_data_stream();
    let mut buf = String::new();
    let deadline = Duration::from_secs(5);

    loop {
        let chunk = tokio::time::timeout(deadline, stream.next()).await;
        let bytes: Bytes = match chunk {
            Ok(Some(Ok(b))) => b,
            Ok(Some(Err(_))) | Ok(None) | Err(_) => break,
        };
        buf.push_str(&String::from_utf8_lossy(&bytes));

        let mut start = 0;
        while let Some(pos) = buf[start..].find('\n') {
            let line_end = start + pos + 1;
            let line = &buf[start..line_end];
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim_start();
                if !data.is_empty()
                    && let Ok(value) = serde_json::from_str::<Value>(data)
                {
                    return value;
                }
            }
            start = line_end;
        }
        buf.drain(..start);
    }

    panic!("no SSE data message received");
}

async fn mcp_initialize(app: &Router, token: &str) -> (String, Value) {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        }
    });
    let request = mcp_request(token, None, body);
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_status(&response, StatusCode::OK);

    let session_id = response
        .headers()
        .get("mcp-session-id")
        .expect("session id header")
        .to_str()
        .unwrap()
        .to_string();
    let message = sse_first_message(response).await;
    (session_id, message)
}

async fn mcp_initialized_notification(app: &Router, token: &str, session_id: &str) {
    let body = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let request = mcp_request(token, Some(session_id), body);
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_status(&response, StatusCode::ACCEPTED);
}

async fn mcp_call(
    app: &Router,
    token: &str,
    session_id: &str,
    method: &str,
    params: Value,
) -> Value {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": method,
        "params": params
    });
    let request = mcp_request(token, Some(session_id), body);
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_status(&response, StatusCode::OK);
    sse_first_message(response).await
}

fn rpc_result(message: &Value) -> &Value {
    message
        .get("result")
        .unwrap_or_else(|| panic!("expected jsonrpc result, got: {message}"))
}

fn text_block_content(result: &Value) -> Value {
    let text = result["content"]
        .as_array()
        .and_then(|c| c.first())
        .and_then(|b| b["text"].as_str())
        .expect("tool result has text block");
    serde_json::from_str(text).expect("tool text is valid json")
}

fn resource_text_content(result: &Value) -> Value {
    let text = result["contents"]
        .as_array()
        .and_then(|c| c.first())
        .and_then(|b| b["text"].as_str())
        .expect("resource result has text content");
    serde_json::from_str(text).expect("resource text is valid json")
}

#[sqlx::test]
async fn mcp_disabled_returns_not_found(pool: PgPool) {
    let (app, client) = setup_app(pool, false).await;
    let (token, _org_id, _channel_id) = setup_user(&client).await;

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        }
    });
    let request = mcp_request(&token, None, body);
    let response = app.oneshot(request).await.expect("router responds");
    assert_status(&response, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn mcp_initialize_returns_server_info(pool: PgPool) {
    let (app, client) = setup_app(pool, true).await;
    let (token, _org_id, _channel_id) = setup_user(&client).await;

    let (session_id, message) = mcp_initialize(&app, &token).await;
    assert!(!session_id.is_empty());

    let result = rpc_result(&message);
    assert_eq!(result["serverInfo"]["name"], "ruckchat-mcp");
    assert!(result["capabilities"]["tools"].as_object().is_some());
    assert!(result["capabilities"]["resources"].as_object().is_some());
}

#[sqlx::test]
async fn mcp_list_and_call_channels_tool(pool: PgPool) {
    let (app, client) = setup_app(pool, true).await;
    let (token, org_id, _channel_id) = setup_user(&client).await;

    let (session_id, _init) = mcp_initialize(&app, &token).await;
    mcp_initialized_notification(&app, &token, &session_id).await;

    let list_response = mcp_call(&app, &token, &session_id, "tools/list", json!({})).await;
    let tools = rpc_result(&list_response)["tools"]
        .as_array()
        .expect("tools array");
    assert!(tools.iter().any(|t| t["name"] == "list_channels"));

    let call_response = mcp_call(
        &app,
        &token,
        &session_id,
        "tools/call",
        json!({
            "name": "list_channels",
            "arguments": { "organization_id": org_id }
        }),
    )
    .await;
    let channels = text_block_content(rpc_result(&call_response))
        .as_array()
        .expect("channels array")
        .clone();
    assert!(channels.iter().any(|c| c["name"] == "general"));
}

#[sqlx::test]
async fn mcp_get_messages_tool(pool: PgPool) {
    let (app, client) = setup_app(pool, true).await;
    let (token, _org_id, channel_id) = setup_user(&client).await;

    let post_response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "hello mcp" })),
        )
        .await;
    assert_status(&post_response, StatusCode::CREATED);

    let (session_id, _init) = mcp_initialize(&app, &token).await;
    mcp_initialized_notification(&app, &token, &session_id).await;

    let response = mcp_call(
        &app,
        &token,
        &session_id,
        "tools/call",
        json!({
            "name": "get_messages",
            "arguments": {
                "conversation_id": channel_id,
                "conversation_type": "channel"
            }
        }),
    )
    .await;
    let content = text_block_content(rpc_result(&response));
    let messages = content.as_array().expect("messages array");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "hello mcp");
}

#[sqlx::test]
async fn mcp_search_messages_tool(pool: PgPool) {
    let (app, client) = setup_app(pool, true).await;
    let (token, org_id, channel_id) = setup_user(&client).await;

    let post_response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "streamable http test message" })),
        )
        .await;
    assert_status(&post_response, StatusCode::CREATED);

    let (session_id, _init) = mcp_initialize(&app, &token).await;
    mcp_initialized_notification(&app, &token, &session_id).await;

    let response = mcp_call(
        &app,
        &token,
        &session_id,
        "tools/call",
        json!({
            "name": "search_messages",
            "arguments": {
                "organization_id": org_id,
                "query": "streamable http"
            }
        }),
    )
    .await;
    let content = text_block_content(rpc_result(&response));
    let results = content.as_array().expect("search results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["content"], "streamable http test message");
}

#[sqlx::test]
async fn mcp_post_message_confirmation_flow(pool: PgPool) {
    let (app, client) = setup_app(pool, true).await;
    let (token, _org_id, channel_id) = setup_user(&client).await;

    let (session_id, _init) = mcp_initialize(&app, &token).await;
    mcp_initialized_notification(&app, &token, &session_id).await;

    let first = mcp_call(
        &app,
        &token,
        &session_id,
        "tools/call",
        json!({
            "name": "post_message",
            "arguments": {
                "conversation_id": channel_id,
                "conversation_type": "channel",
                "content": "first mcp post"
            }
        }),
    )
    .await;
    let text = rpc_result(&first)["content"]
        .as_array()
        .and_then(|c| c.first())
        .and_then(|b| b["text"].as_str())
        .expect("confirmation text");
    assert!(text.contains("Confirmation required"));

    let second = mcp_call(
        &app,
        &token,
        &session_id,
        "tools/call",
        json!({
            "name": "post_message",
            "arguments": {
                "conversation_id": channel_id,
                "conversation_type": "channel",
                "content": "first mcp post",
                "confirmed": true
            }
        }),
    )
    .await;
    let posted = text_block_content(rpc_result(&second));
    assert_eq!(posted["content"], "first mcp post");
}

#[sqlx::test]
async fn mcp_read_organization_and_channel_resources(pool: PgPool) {
    let (app, client) = setup_app(pool, true).await;
    let (token, org_id, channel_id) = setup_user(&client).await;

    let (session_id, _init) = mcp_initialize(&app, &token).await;
    mcp_initialized_notification(&app, &token, &session_id).await;

    let org_response = mcp_call(
        &app,
        &token,
        &session_id,
        "resources/read",
        json!({ "uri": format!("ruckchat://organization/{org_id}") }),
    )
    .await;
    let org = resource_text_content(rpc_result(&org_response));
    assert_eq!(org["id"], org_id);

    let channel_response = mcp_call(
        &app,
        &token,
        &session_id,
        "resources/read",
        json!({ "uri": format!("ruckchat://channel/{channel_id}") }),
    )
    .await;
    let channel = resource_text_content(rpc_result(&channel_response));
    assert_eq!(channel["id"], channel_id);
    assert_eq!(channel["name"], "general");
}
