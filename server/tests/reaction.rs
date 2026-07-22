//! Reaction REST endpoint integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;
use sqlx::PgPool;

async fn register_and_login(client: &TestClient) -> (String, String, String) {
    let email = test_email("reaction");
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

#[sqlx::test]
async fn add_and_remove_reaction(pool: PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = register_and_login(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "react to me" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let message_id = body_json(response).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = client
        .auth_request(
            "POST",
            &format!("/messages/{message_id}/reactions"),
            &token,
            Some(json!({ "emoji": "👍" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let body = body_json(response).await;
    assert_eq!(body["emoji"], "👍");
    assert_eq!(body["message_id"], message_id);

    let response = client
        .auth_request(
            "DELETE",
            &format!("/messages/{message_id}/reactions/%F0%9F%91%8D"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn reaction_forbidden_for_non_member(pool: PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, _org_id, channel_id) = register_and_login(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            Some(json!({ "content": "private" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let message_id = body_json(response).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let outsider_email = test_email("reaction-outsider");
    let response = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": outsider_email,
                "display_name": "Outsider",
                "password": "correct horse battery staple",
                "organization_name": "Other",
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
                "email": outsider_email,
                "password": "correct horse battery staple"
            })),
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let outsider_token = body_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let response = client
        .auth_request(
            "POST",
            &format!("/messages/{message_id}/reactions"),
            &outsider_token,
            Some(json!({ "emoji": "👍" })),
        )
        .await;
    assert_status(&response, StatusCode::FORBIDDEN);
}
