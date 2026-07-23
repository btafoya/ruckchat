//! Web Push subscription integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use ruckchat_config::{AppConfig, DatabaseConfig, Environment, WebConfig, WebPushConfig};
use ruckchat_server::{handlers::router, state::AppState};
use serde_json::json;

fn push_config() -> WebPushConfig {
    WebPushConfig {
        enabled: true,
        subject: Some("mailto:test@example.com".into()),
        vapid_public_key: Some("cHVibGljLWtleQ".into()),
        vapid_private_key: Some("cHJpdmF0ZS1rZXk".into()),
    }
}

async fn client_with_push(pool: sqlx::PgPool) -> (TestClient, String) {
    ruckchat_migrations::migrator()
        .run(&pool)
        .await
        .expect("migrations apply");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL set");
    let config = AppConfig {
        app_name: "RuckChat".into(),
        environment: Environment::Test,
        base_url: "http://localhost:3000".into(),
        log_level: "warn".into(),
        database: DatabaseConfig::from_url(database_url),
        mcp: ruckchat_config::McpConfig::default(),
        plugins: ruckchat_config::PluginConfig::default(),
        files: ruckchat_config::FilesConfig::default(),
        web: WebConfig::default(),
        web_push: push_config(),
    };
    let state = AppState::from_config(pool, &config);
    let client = TestClient::from_router(router(&config.web, &config.base_url).with_state(state));

    let email = test_email("webpush");
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

    (client, token)
}

#[sqlx::test]
async fn vapid_key_returns_public_key(pool: sqlx::PgPool) {
    let (client, _) = client_with_push(pool).await;

    let response = client.request("GET", "/web-push/vapid-key", None).await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["public_key"], "cHVibGljLWtleQ");
}

#[sqlx::test]
async fn subscribe_and_unsubscribe_round_trip(pool: sqlx::PgPool) {
    let (client, token) = client_with_push(pool).await;

    let response = client
        .auth_request(
            "POST",
            "/web-push/subscribe",
            &token,
            Some(json!({
                "endpoint": "https://example.test/push/1",
                "p256dh": "p256dh-key",
                "auth": "auth-secret"
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let response = client
        .auth_request(
            "POST",
            "/web-push/unsubscribe",
            &token,
            Some(json!({
                "endpoint": "https://example.test/push/1"
            })),
        )
        .await;
    assert_status(&response, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn subscribe_returns_forbidden_when_push_disabled(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let email = test_email("webpush-disabled");
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
            "POST",
            "/web-push/subscribe",
            &token,
            Some(json!({
                "endpoint": "https://example.test/push/1",
                "p256dh": "p256dh-key",
                "auth": "auth-secret"
            })),
        )
        .await;
    assert_status(&response, StatusCode::FORBIDDEN);
}
