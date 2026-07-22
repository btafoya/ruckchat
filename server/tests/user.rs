//! User route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

async fn register_and_login(client: &TestClient) -> (String, String) {
    let email = test_email("user");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Alice",
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
    let token = body["token"].as_str().unwrap().to_string();
    (token, email)
}

#[sqlx::test]
async fn get_profile_returns_authenticated_user(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, email) = register_and_login(&client).await;

    let response = client.auth_request("GET", "/users/me", &token, None).await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["email"], email);
    assert!(body["password_hash"].is_null());
}

#[sqlx::test]
async fn update_profile_changes_display_name(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _email) = register_and_login(&client).await;

    let response = client
        .auth_request(
            "PATCH",
            "/users/me",
            &token,
            Some(json!({ "display_name": "Alice Updated" })),
        )
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["display_name"], "Alice Updated");
}
