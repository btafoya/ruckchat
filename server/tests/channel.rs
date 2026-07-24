//! Channel route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

async fn setup_owner(client: &TestClient) -> (String, String) {
    let email = test_email("channel");
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
    (token, org_id)
}

#[sqlx::test]
async fn create_channel(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id) = setup_owner(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/channels"),
            &token,
            Some(json!({ "name": "random", "is_private": false })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["name"], "random");
}

#[sqlx::test]
async fn list_channels(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id) = setup_owner(&client).await;

    client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/channels"),
            &token,
            Some(json!({ "name": "random", "is_private": false })),
        )
        .await;

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
    let channels = body["items"].as_array().unwrap();
    assert_eq!(channels.len(), 2); // general from registration plus random
}

#[sqlx::test]
async fn archive_channel(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id) = setup_owner(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/channels"),
            &token,
            Some(json!({ "name": "temp", "is_private": false })),
        )
        .await;
    let body = body_json(response).await;
    let channel_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request("DELETE", &format!("/channels/{channel_id}"), &token, None)
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert!(!body["archived_at"].is_null());
}

#[sqlx::test]
async fn unarchive_channel(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id) = setup_owner(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/channels"),
            &token,
            Some(json!({ "name": "temp", "is_private": false })),
        )
        .await;
    let body = body_json(response).await;
    let channel_id = body["id"].as_str().unwrap().to_string();

    client
        .auth_request("DELETE", &format!("/channels/{channel_id}"), &token, None)
        .await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/unarchive"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert!(body["archived_at"].is_null());
}

#[sqlx::test]
async fn member_can_create_channel(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id) = setup_owner(&client).await;

    let member_email = test_email("channel-member");
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
            Some(json!({ "email": member_email, "role": "member" })),
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
            &format!("/organizations/{org_id}/channels"),
            &member_token,
            Some(json!({ "name": "member-made", "is_private": false })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
}
