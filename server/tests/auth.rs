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
async fn register_succeeds_when_allowed(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": test_email("register-allowed"),
                "display_name": "Alice",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": "acme-allowed"
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
}

#[sqlx::test]
async fn register_fails_when_disabled(pool: sqlx::PgPool) {
    let client = TestClient::new(pool.clone()).await;

    // Register the first user, who becomes a server admin.
    let admin_email = test_email("register-disabled-admin");
    let admin_password = "correct horse battery staple";
    let register = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": admin_email,
                "display_name": "Admin",
                "password": admin_password,
                "organization_name": "Admin Org",
                "organization_slug": format!("admin-org-{}", uuid::Uuid::new_v4())
            })),
        )
        .await;
    assert_status(&register, StatusCode::CREATED);

    let reg_body = body_json(register).await;
    let org_id = reg_body["organization"]["id"].as_str().unwrap().to_string();

    let login = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({"email": admin_email, "password": admin_password})),
        )
        .await;
    assert_status(&login, StatusCode::OK);
    let login_body = body_json(login).await;
    let admin_token = login_body["token"].as_str().unwrap().to_string();

    // Disable new user registrations.
    let update = client
        .auth_request(
            "PUT",
            "/api/v1/server/settings",
            &admin_token,
            Some(json!({
                "maintenance_mode_enabled": false,
                "default_max_file_size_bytes": 26214400i64,
                "default_storage_quota_bytes": 10737418240i64,
                "allowed_signup_domains": [],
                "allow_registration": false
            })),
        )
        .await;
    assert_status(&update, StatusCode::OK);

    let status = client
        .request("GET", "/auth/registration-status", None)
        .await;
    assert_status(&status, StatusCode::OK);
    let status_body = body_json(status).await;
    assert_eq!(status_body["allow_registration"].as_bool(), Some(false));

    // Public registration should now be rejected.
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": test_email("register-disabled"),
                "display_name": "Bob",
                "password": "correct horse battery staple",
                "organization_name": "Blocked",
                "organization_slug": "blocked"
            })),
        )
        .await;
    assert_status(&response, StatusCode::FORBIDDEN);
    let body = body_json(response).await;
    assert_eq!(body["code"].as_str().unwrap(), "forbidden");

    // Server admin user creation must still work when registration is disabled.
    let created_email = test_email("register-disabled-created");
    let create = client
        .auth_request(
            "POST",
            "/api/v1/server/users",
            &admin_token,
            Some(json!({
                "email": created_email,
                "display_name": "Created User",
                "password": "correct horse battery staple"
            })),
        )
        .await;
    assert_status(&create, StatusCode::CREATED);

    // Organization invites must still work when registration is disabled.
    let invite = client
        .auth_request(
            "POST",
            &format!("/organizations/{}/members", org_id),
            &admin_token,
            Some(json!({
                "email": created_email,
                "role": "member"
            })),
        )
        .await;
    assert_status(&invite, StatusCode::CREATED);
}

#[sqlx::test]
async fn protected_endpoint_requires_authentication(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let response = client.request("GET", "/users/me", None).await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}
