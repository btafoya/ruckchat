//! Message route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

async fn setup_channel(client: &TestClient) -> (String, String, String) {
    let email = test_email("message");
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
async fn post_and_list_messages(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_channel(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "hello world" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    let response = client
        .auth_request(
            "GET",
            &format!("/channels/{channel_id}/messages"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    let messages = body["items"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "hello world");
}

#[sqlx::test]
async fn edit_message(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_channel(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "hello" })),
        )
        .await;
    let body = body_json(response).await;
    let message_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "PATCH",
            &format!("/messages/{message_id}"),
            &token,
            Some(json!({ "content": "hello world" })),
        )
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["content"], "hello world");
}

#[sqlx::test]
async fn delete_message(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_channel(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "delete me" })),
        )
        .await;
    let body = body_json(response).await;
    let message_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request("DELETE", &format!("/messages/{message_id}"), &token, None)
        .await;
    assert_status(&response, StatusCode::NO_CONTENT);
}
