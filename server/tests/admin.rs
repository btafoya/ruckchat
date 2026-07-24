//! Administrative endpoint integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;
use uuid::Uuid;

async fn owner_context(client: &TestClient) -> (String, Uuid, String) {
    let email = test_email("admin-owner");
    let password = "correct horse battery staple";

    let register = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Owner",
                "password": password,
                "organization_name": "Migration Target",
                "organization_slug": "migration-target"
            })),
        )
        .await;
    assert_status(&register, StatusCode::CREATED);
    let reg_body = body_json(register).await;
    let org_id = reg_body["organization"]["id"].as_str().unwrap().to_string();
    let owner_id = reg_body["user"]["id"].as_str().unwrap().to_string();

    let login = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({
                "email": email,
                "password": password
            })),
        )
        .await;
    assert_status(&login, StatusCode::OK);
    let login_body = body_json(login).await;
    let token = login_body["token"].as_str().unwrap().to_string();

    (token, org_id.parse().unwrap(), owner_id)
}

async fn member_context(client: &TestClient, owner_token: &str, org_id: Uuid) -> String {
    let email = test_email("admin-member");
    let password = "correct horse battery staple";

    // Register the user outside the organization first.
    let register = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Member",
                "password": password,
                "organization_name": "Other",
                "organization_slug": format!("other-{}", Uuid::new_v4())
            })),
        )
        .await;
    assert_status(&register, StatusCode::CREATED);

    // Invite them to the target organization as a member.
    let invite = client
        .auth_request(
            "POST",
            &format!("/organizations/{}/members", org_id),
            owner_token,
            Some(json!({
                "email": email,
                "role": "member"
            })),
        )
        .await;
    assert_status(&invite, StatusCode::CREATED);

    let login = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({
                "email": email,
                "password": password
            })),
        )
        .await;
    assert_status(&login, StatusCode::OK);
    let login_body = body_json(login).await;
    login_body["token"].as_str().unwrap().to_string()
}

fn now() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC 3339 format is valid")
}

fn empty_data() -> serde_json::Value {
    json!({
        "version": 2,
        "exported_at": now(),
        "users": [],
        "organizations": [],
        "organization_memberships": [],
        "organization_settings": [],
        "organization_roles": [],
        "permissions": [],
        "role_permissions": [],
        "custom_emoji": [],
        "teams": [],
        "team_memberships": [],
        "team_rooms": [],
        "channels": [],
        "channel_memberships": [],
        "direct_message_conversations": [],
        "messages": [],
        "reactions": [],
        "files": [],
        "message_files": []
    })
}

#[sqlx::test]
async fn import_requires_authentication(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let org_id = Uuid::new_v4();
    let response = client
        .request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/import", org_id),
            Some(json!({"data": empty_data()})),
        )
        .await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn import_dry_run_counts_existing_rows(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, owner_id) = owner_context(&client).await;

    let now = now();
    let mut snapshot = empty_data();
    snapshot["users"] = json!([{
        "id": owner_id,
        "email": "owner@example.com",
        "display_name": "Owner",
        "password_hash": "hash",
        "avatar_url": null,
        "deactivated_at": null,
        "created_at": now,
        "updated_at": now
    }]);
    snapshot["organizations"] = json!([{
        "id": org_id,
        "name": "Migration Target",
        "slug": "migration-target",
        "owner_id": owner_id,
        "created_at": now,
        "updated_at": now
    }]);
    snapshot["organization_memberships"] = json!([{
        "user_id": owner_id,
        "organization_id": org_id,
        "role": "owner",
        "joined_at": now
    }]);
    snapshot["organization_settings"] = json!([{
        "organization_id": org_id,
        "max_file_size_bytes": 10485760,
        "storage_quota_bytes": 10737418240i64,
        "updated_at": now
    }]);

    let response = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/import", org_id),
            &token,
            Some(json!({
                "data": snapshot,
                "dry_run": true
            })),
        )
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert_eq!(body["inserted"].as_u64().unwrap(), 0);
    assert_eq!(body["skipped"].as_u64().unwrap(), 4);
}

#[sqlx::test]
async fn import_applies_snapshot(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, owner_id) = owner_context(&client).await;

    let now = now();
    let new_user_id = Uuid::new_v4();
    let new_user_email = test_email("imported");
    let channel_id = Uuid::new_v4();

    let mut snapshot = empty_data();
    snapshot["users"] = json!([
        {
            "id": owner_id,
            "email": "owner@example.com",
            "display_name": "Owner",
            "password_hash": "hash",
            "avatar_url": null,
            "deactivated_at": null,
            "created_at": now,
            "updated_at": now
        },
        {
            "id": new_user_id,
            "email": new_user_email,
            "display_name": "Imported",
            "password_hash": "hash",
            "avatar_url": null,
            "deactivated_at": null,
            "created_at": now,
            "updated_at": now
        }
    ]);
    snapshot["organizations"] = json!([{
        "id": org_id,
        "name": "Migration Target",
        "slug": "migration-target",
        "owner_id": owner_id,
        "created_at": now,
        "updated_at": now
    }]);
    snapshot["organization_memberships"] = json!([
        {
            "user_id": owner_id,
            "organization_id": org_id,
            "role": "owner",
            "joined_at": now
        },
        {
            "user_id": new_user_id,
            "organization_id": org_id,
            "role": "member",
            "joined_at": now
        }
    ]);
    snapshot["organization_settings"] = json!([{
        "organization_id": org_id,
        "max_file_size_bytes": 10485760,
        "storage_quota_bytes": 10737418240i64,
        "updated_at": now
    }]);
    snapshot["channels"] = json!([{
        "id": channel_id,
        "organization_id": org_id,
        "name": "imported-channel",
        "topic": null,
        "purpose": null,
        "is_private": false,
        "created_by": owner_id,
        "created_at": now,
        "archived_at": null
    }]);

    let response = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/import", org_id),
            &token,
            Some(json!({
                "data": snapshot,
                "dry_run": false
            })),
        )
        .await;
    assert_status(&response, StatusCode::OK);

    let body = body_json(response).await;
    assert!(body["inserted"].as_u64().unwrap() > 0);

    // The new channel should now be readable by the owner.
    let channel = client
        .auth_request("GET", &format!("/channels/{}", channel_id), &token, None)
        .await;
    assert_status(&channel, StatusCode::OK);
}

#[sqlx::test]
async fn import_rejects_snapshot_for_other_organization(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, owner_id) = owner_context(&client).await;

    let now = now();
    let other_org_id = Uuid::new_v4();
    let mut snapshot = empty_data();
    snapshot["users"] = json!([{
        "id": owner_id,
        "email": "owner@example.com",
        "display_name": "Owner",
        "password_hash": "hash",
        "avatar_url": null,
        "deactivated_at": null,
        "created_at": now,
        "updated_at": now
    }]);
    snapshot["organizations"] = json!([{
        "id": other_org_id,
        "name": "Other",
        "slug": "other",
        "owner_id": owner_id,
        "created_at": now,
        "updated_at": now
    }]);

    let response = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/import", org_id),
            &token,
            Some(json!({
                "data": snapshot,
                "dry_run": true
            })),
        )
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn import_forbidden_for_non_admin_member(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, _owner_id) = owner_context(&client).await;
    let member_token = member_context(&client, &owner_token, org_id).await;

    let response = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/import", org_id),
            &member_token,
            Some(json!({
                "data": empty_data(),
                "dry_run": true
            })),
        )
        .await;
    assert_status(&response, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn list_and_create_roles(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, _owner_id) = owner_context(&client).await;

    let create = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/roles", org_id),
            &token,
            Some(json!({
                "name": "moderator",
                "description": "Can moderate messages"
            })),
        )
        .await;
    assert_status(&create, StatusCode::CREATED);

    let list = client
        .auth_request(
            "GET",
            &format!("/api/v1/admin/organizations/{}/roles", org_id),
            &token,
            None,
        )
        .await;
    assert_status(&list, StatusCode::OK);
    let body = body_json(list).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "moderator");
}

#[sqlx::test]
async fn list_and_create_permissions(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, _owner_id) = owner_context(&client).await;

    let create = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/permissions", org_id),
            &token,
            Some(json!({
                "key": "manage_channels",
                "description": "Create and archive channels"
            })),
        )
        .await;
    assert_status(&create, StatusCode::CREATED);

    let list = client
        .auth_request(
            "GET",
            &format!("/api/v1/admin/organizations/{}/permissions", org_id),
            &token,
            None,
        )
        .await;
    assert_status(&list, StatusCode::OK);
    let body = body_json(list).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["key"], "manage_channels");
}

#[sqlx::test]
async fn list_and_create_teams(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, _owner_id) = owner_context(&client).await;

    let create = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/teams", org_id),
            &token,
            Some(json!({
                "name": "Engineering",
                "description": "The engineering team"
            })),
        )
        .await;
    assert_status(&create, StatusCode::CREATED);

    let list = client
        .auth_request(
            "GET",
            &format!("/api/v1/admin/organizations/{}/teams", org_id),
            &token,
            None,
        )
        .await;
    assert_status(&list, StatusCode::OK);
    let body = body_json(list).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Engineering");
}

#[sqlx::test]
async fn list_and_create_emoji(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id, _owner_id) = owner_context(&client).await;

    // Record a file to reference as the emoji image.
    let file = client
        .auth_request(
            "POST",
            "/files/record",
            &token,
            Some(json!({
                "organization_id": org_id,
                "file_name": "emoji.png",
                "mime_type": "image/png",
                "size_bytes": 1024,
                "storage_path": "/files/emoji.png"
            })),
        )
        .await;
    assert_status(&file, StatusCode::CREATED);
    let file_body = body_json(file).await;
    let file_id = file_body["id"].as_str().unwrap();

    let create = client
        .auth_request(
            "POST",
            &format!("/api/v1/admin/organizations/{}/emoji", org_id),
            &token,
            Some(json!({
                "shortcode": "partyparrot",
                "file_id": file_id
            })),
        )
        .await;
    assert_status(&create, StatusCode::CREATED);

    let list = client
        .auth_request(
            "GET",
            &format!("/api/v1/admin/organizations/{}/emoji", org_id),
            &token,
            None,
        )
        .await;
    assert_status(&list, StatusCode::OK);
    let body = body_json(list).await;
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["shortcode"], "partyparrot");
}

#[sqlx::test]
async fn admin_create_endpoints_forbidden_for_non_admin(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (owner_token, org_id, _owner_id) = owner_context(&client).await;
    let member_token = member_context(&client, &owner_token, org_id).await;

    let cases = [
        (
            format!("/api/v1/admin/organizations/{}/roles", org_id),
            json!({"name": "x"}),
        ),
        (
            format!("/api/v1/admin/organizations/{}/permissions", org_id),
            json!({"key": "x"}),
        ),
        (
            format!("/api/v1/admin/organizations/{}/teams", org_id),
            json!({"name": "x"}),
        ),
    ];
    for (path, body) in cases {
        let response = client
            .auth_request("POST", &path, &member_token, Some(body))
            .await;
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "path {path} should be forbidden"
        );
    }
}
