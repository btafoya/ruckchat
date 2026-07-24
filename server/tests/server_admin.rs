//! Server-wide administrative endpoint integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn server_admin_context(client: &TestClient, pool: &PgPool) -> (String, Uuid, Uuid) {
    let email = test_email("server-admin");
    let password = "correct horse battery staple";

    let register = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Server Admin",
                "password": password,
                "organization_name": "Admin Org",
                "organization_slug": format!("admin-org-{}", Uuid::new_v4())
            })),
        )
        .await;
    assert_status(&register, StatusCode::CREATED);
    let reg_body = body_json(register).await;
    let org_id = reg_body["organization"]["id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .unwrap();
    let admin_id = reg_body["user"]["id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .unwrap();

    sqlx::query!(
        "UPDATE users SET is_server_admin = true WHERE id = $1",
        admin_id
    )
    .execute(pool)
    .await
    .expect("mark user as server admin");

    let login = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({"email": email, "password": password})),
        )
        .await;
    assert_status(&login, StatusCode::OK);
    let login_body = body_json(login).await;
    let token = login_body["token"].as_str().unwrap().to_string();

    (token, org_id, admin_id)
}

async fn regular_user_context(client: &TestClient) -> (String, Uuid, String) {
    let email = test_email("regular-user");
    let password = "correct horse battery staple";

    let register = client
        .request(
            "POST",
            "/auth/register",
            Some(json!({
                "email": email,
                "display_name": "Regular User",
                "password": password,
                "organization_name": "User Org",
                "organization_slug": format!("user-org-{}", Uuid::new_v4())
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
    let login_body = body_json(login).await;
    let token = login_body["token"].as_str().unwrap().to_string();
    let user_id = login_body["user"]["id"].as_str().unwrap().parse().unwrap();

    (token, user_id, email)
}

#[sqlx::test]
async fn server_admin_endpoints_require_admin(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (_admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;
    let (token, _user_id, _email) = regular_user_context(&client).await;

    let paths = [
        "GET:/api/v1/server/organizations",
        "GET:/api/v1/server/users",
        "GET:/api/v1/server/admins",
        "GET:/api/v1/server/settings",
        "GET:/api/v1/server/audit-log",
    ];
    for spec in paths {
        let (method, path) = spec.split_once(':').unwrap();
        let response = client.auth_request(method, path, &token, None).await;
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "{method} {path} should require server admin"
        );
    }
}

#[sqlx::test]
async fn server_admin_can_list_and_update_user(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;
    let (_user_token, user_id, _email) = regular_user_context(&client).await;

    let list = client
        .auth_request("GET", "/api/v1/server/users", &admin_token, None)
        .await;
    assert_status(&list, StatusCode::OK);
    let list_body = body_json(list).await;
    assert!(list_body["items"].as_array().unwrap().len() >= 2);

    let get = client
        .auth_request(
            "GET",
            &format!("/api/v1/server/users/{}", user_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&get, StatusCode::OK);
    let get_body = body_json(get).await;
    assert_eq!(get_body["id"].as_str().unwrap(), user_id.to_string());

    let update = client
        .auth_request(
            "PATCH",
            &format!("/api/v1/server/users/{}", user_id),
            &admin_token,
            Some(json!({
                "display_name": "Updated Name",
                "email": test_email("updated-user")
            })),
        )
        .await;
    assert_status(&update, StatusCode::OK);
    let update_body = body_json(update).await;
    assert_eq!(update_body["display_name"], "Updated Name");
}

#[sqlx::test]
async fn server_admin_can_promote_and_demote_user(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, admin_id) = server_admin_context(&client, &pool).await;
    let (_user_token, user_id, _email) = regular_user_context(&client).await;

    let demote_self = client
        .auth_request(
            "POST",
            &format!("/api/v1/server/users/{}/demote", admin_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&demote_self, StatusCode::BAD_REQUEST);

    let promote = client
        .auth_request(
            "POST",
            &format!("/api/v1/server/users/{}/promote", user_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&promote, StatusCode::OK);

    let admins = client
        .auth_request("GET", "/api/v1/server/admins", &admin_token, None)
        .await;
    assert_status(&admins, StatusCode::OK);
    let admins_body = body_json(admins).await;
    let admin_ids: Vec<_> = admins_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["id"].as_str().unwrap().to_string())
        .collect();
    assert!(admin_ids.contains(&user_id.to_string()));

    let demote = client
        .auth_request(
            "POST",
            &format!("/api/v1/server/users/{}/demote", user_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&demote, StatusCode::OK);
}

#[sqlx::test]
async fn server_admin_can_reset_password(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;
    let (_user_token, user_id, email) = regular_user_context(&client).await;

    let reset = client
        .auth_request(
            "POST",
            &format!("/api/v1/server/users/{}/reset-password", user_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&reset, StatusCode::OK);
    let reset_body = body_json(reset).await;
    let new_password = reset_body["password"].as_str().unwrap();

    let login = client
        .request(
            "POST",
            "/auth/login",
            Some(json!({"email": email, "password": new_password})),
        )
        .await;
    assert_status(&login, StatusCode::OK);
}

#[sqlx::test]
async fn server_admin_can_deactivate_and_reactivate_user(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;
    let (_user_token, user_id, _email) = regular_user_context(&client).await;

    let deactivate = client
        .auth_request(
            "POST",
            &format!("/api/v1/server/users/{}/deactivate", user_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&deactivate, StatusCode::OK);
    let body = body_json(deactivate).await;
    assert!(body["deactivated_at"].is_string());

    let reactivate = client
        .auth_request(
            "POST",
            &format!("/api/v1/server/users/{}/reactivate", user_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&reactivate, StatusCode::OK);
    let body = body_json(reactivate).await;
    assert!(body["deactivated_at"].is_null());
}

#[sqlx::test]
async fn server_admin_can_manage_organizations(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;

    let create = client
        .auth_request(
            "POST",
            "/api/v1/server/organizations",
            &admin_token,
            Some(json!({
                "name": "Created Org",
                "slug": format!("created-org-{}", Uuid::new_v4())
            })),
        )
        .await;
    assert_status(&create, StatusCode::CREATED);
    let create_body = body_json(create).await;
    let org_id = create_body["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let rename = client
        .auth_request(
            "PATCH",
            &format!("/api/v1/server/organizations/{}", org_id),
            &admin_token,
            Some(json!({"name": "Renamed Org"})),
        )
        .await;
    assert_status(&rename, StatusCode::OK);
    let rename_body = body_json(rename).await;
    assert_eq!(rename_body["name"], "Renamed Org");

    let delete = client
        .auth_request(
            "DELETE",
            &format!("/api/v1/server/organizations/{}", org_id),
            &admin_token,
            None,
        )
        .await;
    assert_status(&delete, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn server_settings_get_and_update(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;

    let get = client
        .auth_request("GET", "/api/v1/server/settings", &admin_token, None)
        .await;
    assert_status(&get, StatusCode::OK);
    let get_body = body_json(get).await;
    assert!(!get_body["maintenance_mode_enabled"].as_bool().unwrap());

    let update = client
        .auth_request(
            "PUT",
            "/api/v1/server/settings",
            &admin_token,
            Some(json!({
                "maintenance_mode_enabled": true,
                "default_max_file_size_bytes": 5242880i64,
                "default_storage_quota_bytes": 5368709120i64,
                "allowed_signup_domains": ["example.com"],
                "allow_registration": false,
                "spelling_enabled": false,
                "spelling_default_language": "en-US"
            })),
        )
        .await;
    assert_status(&update, StatusCode::OK);

    let get = client
        .auth_request("GET", "/api/v1/server/settings", &admin_token, None)
        .await;
    assert_status(&get, StatusCode::OK);
    let get_body = body_json(get).await;
    assert!(get_body["maintenance_mode_enabled"].as_bool().unwrap());
    assert_eq!(get_body["allowed_signup_domains"], json![["example.com"]]);
    assert!(!get_body["allow_registration"].as_bool().unwrap());
    assert!(!get_body["spelling_enabled"].as_bool().unwrap());
    assert_eq!(get_body["spelling_default_language"], "en-US");
}

#[sqlx::test]
async fn audit_log_records_admin_action(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;
    let (_user_token, user_id, _email) = regular_user_context(&client).await;

    let update = client
        .auth_request(
            "PATCH",
            &format!("/api/v1/server/users/{}", user_id),
            &admin_token,
            Some(json!({"display_name": "Audited Name"})),
        )
        .await;
    assert_status(&update, StatusCode::OK);

    let log = client
        .auth_request("GET", "/api/v1/server/audit-log", &admin_token, None)
        .await;
    assert_status(&log, StatusCode::OK);
    let log_body = body_json(log).await;
    let actions: Vec<_> = log_body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap().to_string())
        .collect();
    assert!(actions.contains(&"user.updated".to_string()));
}

#[sqlx::test]
async fn impersonation_start_and_end(pool: PgPool) {
    let client = TestClient::new(pool.clone()).await;
    let (admin_token, _org_id, _admin_id) = server_admin_context(&client, &pool).await;
    let (_user_token, user_id, _email) = regular_user_context(&client).await;

    let start = client
        .auth_request(
            "POST",
            "/api/v1/server/impersonate",
            &admin_token,
            Some(json!({"target_user_id": user_id})),
        )
        .await;
    assert_status(&start, StatusCode::OK);
    let start_body = body_json(start).await;
    let token = start_body["token"].as_str().unwrap();

    let end = client
        .auth_request(
            "DELETE",
            "/api/v1/server/impersonate",
            &admin_token,
            Some(json!({"token": token})),
        )
        .await;
    assert_status(&end, StatusCode::NO_CONTENT);
}
