//! Authentication route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

#[sqlx::test]
async fn register_creates_user_and_organization(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let email = test_email("register");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Alice",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": "acme"
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["user"]["email"], email);
    assert_eq!(body["organization"]["slug"], "acme");
    assert!(body["user"]["password_hash"].is_null());
}

#[sqlx::test]
async fn register_rejects_short_password(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": test_email("short"),
                "display_name": "Bob",
                "password": "short",
                "organization_name": "Acme",
                "organization_slug": "acme-short"
            })),
        )
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn login_succeeds_with_valid_credentials(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let email = test_email("login");
    client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Alice",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": "acme-login"
            })),
        )
        .await;

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
    assert!(!body["token"].as_str().unwrap().is_empty());
    assert_eq!(body["user"]["email"], email);
}

#[sqlx::test]
async fn login_fails_with_wrong_password(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let email = test_email("login-wrong");
    client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Alice",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": "acme-login-wrong"
            })),
        )
        .await;

    let response = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({
                "email": email,
                "password": "wrong password"
            })),
        )
        .await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn protected_endpoint_requires_authentication(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let response = client.request("GET", "/users/me", None).await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}
