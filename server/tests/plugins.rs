//! Plugin command route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;
use uuid::Uuid;

async fn setup_user(client: &TestClient) -> String {
    let email = test_email("plugin");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "User",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": uuid::Uuid::new_v4().to_string()
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

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
    body["token"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn missing_plugin_command_returns_not_found(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let token = setup_user(&client).await;

    let response = client
        .auth_request(
            "POST",
            "/plugins/missing/commands/hello",
            &token,
            Some(json!({
                "conversation_id": Uuid::new_v4(),
                "conversation_type": "channel",
                "args": ["world"]
            })),
        )
        .await;
    assert_status(&response, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn plugin_command_requires_authentication(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;

    let response = client
        .request(
            "POST",
            "/plugins/echo/commands/hello",
            Some(json!({
                "conversation_id": Uuid::new_v4(),
                "conversation_type": "channel",
                "args": []
            })),
        )
        .await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}
