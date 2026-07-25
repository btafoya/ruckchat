//! Cursor-based message pagination integration tests (ADR-016).

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

/// Registers an owner with a fresh organization and its default channel.
async fn setup_owner(client: &TestClient) -> (String, String, String) {
    let email = test_email("pagination");
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

async fn post_message(client: &TestClient, token: &str, channel_id: &str, content: &str) -> String {
    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            token,
            Some(json!({ "content": content })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    body_json(response).await["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[sqlx::test]
async fn history_defaults_to_newest_page_in_ascending_order(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_owner(&client).await;

    let mut ids = Vec::new();
    for i in 0..5 {
        ids.push(post_message(&client, &token, &channel_id, &format!("msg-{i}")).await);
    }

    let response = client
        .auth_request(
            "GET",
            &format!("/channels/{channel_id}/messages?limit=2"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Ascending order: the two newest messages, oldest of the pair first.
    assert_eq!(items[0]["id"], ids[3]);
    assert_eq!(items[1]["id"], ids[4]);
    assert_eq!(body["has_more_older"], true);
}

#[sqlx::test]
async fn before_id_pages_backward_through_history(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_owner(&client).await;

    let mut ids = Vec::new();
    for i in 0..5 {
        ids.push(post_message(&client, &token, &channel_id, &format!("msg-{i}")).await);
    }

    // Load the newest 2 (msg-3, msg-4), then page backward from msg-3.
    let response = client
        .auth_request(
            "GET",
            &format!(
                "/channels/{channel_id}/messages?limit=2&before_id={}",
                ids[3]
            ),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], ids[1]);
    assert_eq!(items[1]["id"], ids[2]);
    assert_eq!(body["has_more_older"], true);

    // Page backward once more; only msg-0 remains older than msg-1.
    let response = client
        .auth_request(
            "GET",
            &format!(
                "/channels/{channel_id}/messages?limit=2&before_id={}",
                ids[1]
            ),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], ids[0]);
    assert_eq!(body["has_more_older"], false);
}

#[sqlx::test]
async fn after_id_pages_forward_from_an_anchor(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_owner(&client).await;

    let mut ids = Vec::new();
    for i in 0..5 {
        ids.push(post_message(&client, &token, &channel_id, &format!("msg-{i}")).await);
    }

    let response = client
        .auth_request(
            "GET",
            &format!(
                "/channels/{channel_id}/messages?limit=2&after_id={}",
                ids[0]
            ),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], ids[1]);
    assert_eq!(items[1]["id"], ids[2]);
    assert_eq!(body["has_more_newer"], true);

    // limit=3 here (not 2) so the 2 truly remaining messages come back
    // strictly under the requested limit, giving an unambiguous
    // has_more_newer=false rather than hitting the exact-limit boundary.
    let response = client
        .auth_request(
            "GET",
            &format!(
                "/channels/{channel_id}/messages?limit=3&after_id={}",
                ids[2]
            ),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], ids[3]);
    assert_eq!(items[1]["id"], ids[4]);
    assert_eq!(body["has_more_newer"], false);
}

#[sqlx::test]
async fn around_id_returns_both_directions_from_an_anchor(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_owner(&client).await;

    // 7 messages, anchored on the middle one (index 3), with 3 genuinely
    // older and 3 genuinely newer than the anchor on either side.
    let mut ids = Vec::new();
    for i in 0..7 {
        ids.push(post_message(&client, &token, &channel_id, &format!("msg-{i}")).await);
    }

    // limit=4 -> half=2 older + anchor + half=2 newer, leaving one more
    // message unfetched on each side (msg-0 and msg-6).
    let response = client
        .auth_request(
            "GET",
            &format!(
                "/channels/{channel_id}/messages?limit=4&around_id={}",
                ids[3]
            ),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    let returned_ids: Vec<&str> = items.iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert_eq!(
        returned_ids,
        vec![
            ids[1].as_str(),
            ids[2].as_str(),
            ids[3].as_str(),
            ids[4].as_str(),
            ids[5].as_str()
        ]
    );
    assert_eq!(body["has_more_older"], true);
    assert_eq!(body["has_more_newer"], true);
}

#[sqlx::test]
async fn is_unread_excludes_own_messages_and_clears_on_mark_read(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, channel_id) = setup_owner(&client).await;
    let member_token = invite_member(&client, &owner_token, &org_id, "unread-page").await;

    let message_id = post_message(&client, &owner_token, &channel_id, "hello").await;

    // The author never sees their own message as unread.
    let response = client
        .auth_request(
            "GET",
            &format!("/channels/{channel_id}/messages"),
            &owner_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items[0]["is_unread"], false);

    // A different member sees it as unread until they mark it read.
    let response = client
        .auth_request(
            "GET",
            &format!("/channels/{channel_id}/messages"),
            &member_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items[0]["is_unread"], true);

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
            &format!("/channels/{channel_id}/messages"),
            &member_token,
            None,
        )
        .await;
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items[0]["is_unread"], false);
}

#[sqlx::test]
async fn thread_replies_paginate_with_cursors(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, _org_id, channel_id) = setup_owner(&client).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            &token,
            Some(json!({ "content": "parent" })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let parent_id = body_json(response).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut reply_ids = Vec::new();
    for i in 0..4 {
        let response = client
            .auth_request(
                "POST",
                &format!("/channels/{channel_id}/messages"),
                &token,
                Some(json!({ "content": format!("reply-{i}"), "parent_id": parent_id })),
            )
            .await;
        assert_status(&response, StatusCode::CREATED);
        reply_ids.push(
            body_json(response).await["id"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    }

    let response = client
        .auth_request(
            "GET",
            &format!("/messages/{parent_id}/replies?limit=2"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], reply_ids[2]);
    assert_eq!(items[1]["id"], reply_ids[3]);
    assert_eq!(body["has_more_older"], true);

    // limit=3 here (not 2) so the 2 truly remaining replies come back
    // strictly under the requested limit, giving an unambiguous
    // has_more_older=false rather than hitting the exact-limit boundary.
    let response = client
        .auth_request(
            "GET",
            &format!(
                "/messages/{parent_id}/replies?limit=3&before_id={}",
                reply_ids[2]
            ),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], reply_ids[0]);
    assert_eq!(items[1]["id"], reply_ids[1]);
    assert_eq!(body["has_more_older"], false);
}
