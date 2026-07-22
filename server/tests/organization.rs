//! Organization route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

async fn register_and_login(client: &TestClient, slug: &str) -> (String, String) {
    let email = test_email("org");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Owner",
                "password": "correct horse battery staple",
                "organization_name": "Acme",
                "organization_slug": slug
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
    (token, org_id)
}

#[sqlx::test]
async fn create_organization(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id) = register_and_login(&client, "acme-create").await;

    let response = client
        .auth_request(
            "POST",
            "/organizations",
            &token,
            Some(json!({
                "name": "Second Org",
                "slug": uuid::Uuid::new_v4().to_string()
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["name"], "Second Org");
}

#[sqlx::test]
async fn list_organizations(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id) = register_and_login(&client, "acme-list").await;

    let response = client
        .auth_request("GET", "/organizations", &token, None)
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn invite_member(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id) = register_and_login(&client, "acme-invite").await;

    let member_email = test_email("org-member");
    client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": member_email,
                "display_name": "Member",
                "password": "correct horse battery staple",
                "organization_name": "Other",
                "organization_slug": uuid::Uuid::new_v4().to_string()
            })),
        )
        .await;

    let response = client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/members"),
            &owner_token,
            Some(json!({
                "email": member_email,
                "role": "member"
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
}

#[sqlx::test]
async fn member_cannot_invite(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id) = register_and_login(&client, "acme-member-invite").await;

    let member_email = test_email("org-member-invite");
    client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": member_email,
                "display_name": "Member",
                "password": "correct horse battery staple",
                "organization_name": "Other",
                "organization_slug": uuid::Uuid::new_v4().to_string()
            })),
        )
        .await;

    client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/members"),
            &owner_token,
            Some(json!({
                "email": member_email,
                "role": "member"
            })),
        )
        .await;

    let response = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({
                "email": member_email,
                "password": "correct horse battery staple"
            })),
        )
        .await;
    let body = body_json(response).await;
    let member_token = body["token"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/members"),
            &member_token,
            Some(json!({
                "email": test_email("target"),
                "role": "member"
            })),
        )
        .await;
    assert_status(&response, StatusCode::FORBIDDEN);
}
