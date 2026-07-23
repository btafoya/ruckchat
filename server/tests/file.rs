//! File route integration tests.

mod common;

use axum::http::StatusCode;
use common::{TestClient, assert_status, body_json, test_email};
use serde_json::json;

async fn setup(pool: &TestClient) -> (String, String) {
    let email = test_email("file");
    let response = pool
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

    let response = pool
        .request(
            "POST",
            "/auth/login",
            Some(json!({
                "email": email,
                "password": "correct horse battery staple"
            })),
        )
        .await;
    let body = body_json(response).await;
    let token = body["token"].as_str().unwrap().to_string();
    (token, org_id)
}

#[sqlx::test]
async fn record_and_list_files(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id) = setup(&client).await;

    let response = client
        .auth_request(
            "POST",
            "/files/record",
            &token,
            Some(json!({
                "organization_id": org_id,
                "file_name": "report.pdf",
                "mime_type": "application/pdf",
                "size_bytes": 1024,
                "storage_path": "orgs/uuid/report.pdf"
            })),
        )
        .await;
    assert_status(&response, StatusCode::CREATED);
    let body = body_json(response).await;
    let file_id = body["id"].as_str().unwrap().to_string();

    let response = client
        .auth_request(
            "GET",
            &format!("/files?organization_id={org_id}"),
            &token,
            None,
        )
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);

    let response = client
        .auth_request("GET", &format!("/files/{file_id}"), &token, None)
        .await;
    assert_status(&response, StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["file_name"], "report.pdf");
}

#[sqlx::test]
async fn multipart_upload_stores_file(pool: sqlx::PgPool) {
    let client = TestClient::new(pool).await;
    let (token, org_id) = setup(&client).await;

    let boundary = "----WebKitFormBoundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"organization_id\"\r\n\r\n\
         {org_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
         Content-Type: text/plain\r\n\r\n\
         hello\r\n\
         --{boundary}--\r\n"
    )
    .into_bytes();
    let content_type = format!("multipart/form-data; boundary={boundary}");

    let response = client
        .auth_multipart("POST", "/files", &token, &content_type, body)
        .await;
    assert_status(&response, StatusCode::CREATED);
    let body = body_json(response).await;
    assert_eq!(body["file_name"], "test.txt");
    assert_eq!(body["mime_type"], "text/plain");
    assert_eq!(body["size_bytes"], 5);
}
