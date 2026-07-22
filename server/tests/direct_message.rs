//! Direct message route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

async fn setup_two_members(client: &TestClient) -> (String, String, String, String) {
    let owner_email = test_email("dm-owner");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": owner_email,
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
                "email": owner_email,
                "password": "correct horse battery staple"
            })),
        )
        .await;
    let body = body_json(response).await;
    let owner_token = body["token"].as_str().unwrap().to_string();

    let member_email = test_email("dm-member");
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
    let member_id = body["user"]["id"].as_str().unwrap().to_string();

    (owner_token, member_token, member_id, org_id)
}

#[sqlx::test]
async fn start_dm_and_post_message(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, _member_token, member_id, org_id) = setup_two_members(&client).await;

    let response = client
        .auth_request(
            "POST",
            "/direct_messages",
            &owner_token,
            Some(json!({
                "organization_id": org_id,
                "member_ids": [member_id]
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let body = body_json(response).await;
    let conversation_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "POST",
            &format!("/direct_messages/{conversation_id}/messages"),
            &owner_token,
            Some(json!({ "content": "hello there" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["content"], "hello there");
}
