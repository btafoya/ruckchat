//! Search and read-state route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

/// Registers an owner with a fresh organization and its default channel.
async fn setup_owner(client: &TestClient) -> (String, String, String) {
    let email = test_email("search");
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

/// Registers a second user in their own organization, then invites them into
/// `org_id` as a member, returning their session token.
async fn invite_member(client: &TestClient, owner_token: &str, org_id: &str, seed: &str) -> String {
    let email = test_email(seed);
    client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
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
            owner_token,
            Some(json!({ "email": email, "role": "member" })),
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
    body["token"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn unread_counts_and_mark_read_round_trip(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, channel_id) = setup_owner(&client).await;
    let member_token = invite_member(&client, &owner_token, &org_id, "read-member").await;

    // The member must join the channel to receive its messages/badges.
    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/channels"),
            &member_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    let member_channel_id = body["items"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(member_channel_id, channel_id);

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            Some(json!({ "content": "hello everyone" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let body = body_json(response).await;
    let message_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/unread_counts"),
            &member_token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["counts"][&channel_id], 1);

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/read"),
            &member_token,
            Some(json!({ "message_ids": [message_id] })),
        )
        .await;
    assert_status(&response, StatusCode::NO_CONTENT);

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/unread_counts"),
            &member_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    assert!(body["counts"].get(&channel_id).is_none());
}

#[sqlx::test]
async fn own_messages_never_count_as_unread(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, channel_id) = setup_owner(&client).await;

    client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            Some(json!({ "content": "note to self" })),
        )
        .await;

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/unread_counts"),
            &owner_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    assert!(body["counts"].get(&channel_id).is_none());
}

#[sqlx::test]
async fn search_finds_message_channel_person_and_file(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, channel_id) = setup_owner(&client).await;

    client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            Some(json!({ "content": "the quokka escaped again" })),
        )
        .await;

    let response = client
        .auth_request(
            "POST",
            &format!("/organizations/{org_id}/channels"),
            &owner_token,
            Some(json!({ "name": "quokka-watch", "is_private": false })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);

    client
        .auth_request(
            "POST",
            "/files/record",
            &owner_token,
            Some(json!({
                "organization_id": org_id,
                "file_name": "quokka-photo.png",
                "mime_type": "image/png",
                "size_bytes": 1024,
                "storage_path": "orgs/uuid/quokka-photo.png"
            })),
        )
        .await;

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/search?q=quokka"),
            &owner_token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;

    assert_eq!(body["messages"].as_array().unwrap().len(), 1);
    assert_eq!(body["messages"][0]["content"], "the quokka escaped again");
    assert!(
        body["channels"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["name"] == "quokka-watch")
    );
    assert_eq!(body["files"].as_array().unwrap().len(), 1);
    assert_eq!(body["files"][0]["file_name"], "quokka-photo.png");

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/search?q=Owner"),
            &owner_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    assert!(
        body["people"]
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u["display_name"] == "Owner")
    );
    // People results must never leak the password hash.
    assert!(body["people"][0].get("password_hash").is_none());
}

#[sqlx::test]
async fn search_from_operator_filters_by_author(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, channel_id) = setup_owner(&client).await;
    let member_token = invite_member(&client, &owner_token, &org_id, "search-member").await;

    client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            Some(json!({ "content": "deploy from owner" })),
        )
        .await;
    client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &member_token,
            Some(json!({ "content": "deploy from member" })),
        )
        .await;

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/search?q=deploy+from:Member"),
            &owner_token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "deploy from member");
}

#[sqlx::test]
async fn search_is_unread_operator_filters_correctly(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, channel_id) = setup_owner(&client).await;
    let member_token = invite_member(&client, &owner_token, &org_id, "unread-search").await;
    client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/channels"),
            &member_token,
            None,
        )
        .await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            Some(json!({ "content": "unread search target" })),
        )
        .await;
    let body = body_json(response).await;
    let message_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/search?q=is:unread+target"),
            &member_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    assert_eq!(body["messages"].as_array().unwrap().len(), 1);

    client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/read"),
            &member_token,
            Some(json!({ "message_ids": [message_id] })),
        )
        .await;

    let response = client
        .auth_request(
            "GET",
            &format!("/organizations/{org_id}/search?q=is:unread+target"),
            &member_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    assert_eq!(body["messages"].as_array().unwrap().len(), 0);
}
