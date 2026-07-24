//! Spell-checker endpoint integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;
use sqlx::PgPool;

async fn authenticated_client(pool: PgPool) -> (TestClient, String) {
    let client = TestClient::new(pool).await;
    let email = test_email("speller");
    let password = "correct horse battery staple";

    let register = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Speller",
                "password": password,
                "organization_name": "Speller Org",
                "organization_slug": format!("speller-org-{}", uuid::Uuid::new_v4())
            })),
        )
        .await;
    assert_status(&register, StatusCode::CREATED);

    let login = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({"email": email, "password": password})),
        )
        .await;
    assert_status(&login, StatusCode::OK);
    let body = body_json(login).await;
    let token = body["token"].as_str().unwrap().to_string();

    (client, token)
}

#[sqlx::test]
async fn check_reports_misspellings_with_suggestions(pool: PgPool) {
    let (client, token) = authenticated_client(pool).await;

    let response = client
        .auth_request(
            "POST",
            "/api/v1/spelling/check",
            &token,
            Some(json!({"text": "The quikc brown foxx jumps.", "max_suggestions": 3})),
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let misspellings = body["misspellings"].as_array().unwrap();
    let words: Vec<&str> = misspellings
        .iter()
        .map(|m| m["word"].as_str().unwrap())
        .collect();
    assert!(words.contains(&"quikc"));
    assert!(words.contains(&"foxx"));
}

#[sqlx::test]
async fn check_returns_empty_for_correct_text(pool: PgPool) {
    let (client, token) = authenticated_client(pool).await;

    let response = client
        .auth_request(
            "POST",
            "/api/v1/spelling/check",
            &token,
            Some(json!({"text": "Hello world"})),
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    assert!(body["misspellings"].as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn suggest_returns_corrections(pool: PgPool) {
    let (client, token) = authenticated_client(pool).await;

    let response = client
        .auth_request(
            "POST",
            "/api/v1/spelling/suggest",
            &token,
            Some(json!({"word": "foxx", "max": 5})),
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let suggestions: Vec<&str> = body["suggestions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert!(suggestions.contains(&"fox") || suggestions.contains(&"foxy"));
}

#[sqlx::test]
async fn languages_lists_en_us(pool: PgPool) {
    let (client, token) = authenticated_client(pool).await;

    let response = client
        .auth_request("GET", "/api/v1/spelling/languages", &token, None)
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let languages: Vec<&str> = body["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect();
    assert!(languages.contains(&"en-US"));
}

#[sqlx::test]
async fn check_requires_authentication(pool: PgPool) {
    let client = TestClient::new(pool).await;

    let response = client
        .request(
            "POST",
            "/api/v1/spelling/check",
            Some(json!({"text": "Hello world"})),
        )
        .await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn check_returns_empty_when_disabled_via_settings(pool: PgPool) {
    let (client, token) = authenticated_client(pool.clone()).await;

    sqlx::query!(
        "INSERT INTO server_settings (key, value, updated_at)
         VALUES ('spelling_enabled', 'false', NOW())
         ON CONFLICT (key) DO UPDATE SET value = 'false', updated_at = NOW()"
    )
    .execute(&pool)
    .await
    .expect("disable spelling");

    let response = client
        .auth_request(
            "POST",
            "/api/v1/spelling/check",
            &token,
            Some(json!({"text": "quikc foxx"})),
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    assert!(body["misspellings"].as_array().unwrap().is_empty());
}
