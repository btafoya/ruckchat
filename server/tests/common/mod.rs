//! Shared helpers for HTTP integration tests.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use ruckchat_server::{handlers::router, state::AppState};
use sqlx::PgPool;
use tower::ServiceExt;

/// Test client wrapping an Axum router.
#[derive(Clone)]
pub struct TestClient {
    router: Router,
}

#[allow(dead_code)]
impl TestClient {
    /// Creates a test client from a database pool, applying pending migrations.
    pub async fn new(pool: PgPool) -> Self {
        ruckchat_migrations::migrator()
            .run(&pool)
            .await
            .expect("migrations apply");
        let state = AppState::from_pool(pool, false);
        Self {
            router: router().with_state(state),
        }
    }

    /// Sends a request without authentication.
    pub async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Response {
        let mut builder = Request::builder().method(method).uri(path);
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        let request = builder
            .body(Body::from(
                body.map(|b| serde_json::to_string(&b).expect("valid json"))
                    .unwrap_or_default(),
            ))
            .expect("valid request");
        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("router responds")
    }

    /// Sends a request with a bearer token.
    pub async fn auth_request(
        &self,
        method: &str,
        path: &str,
        token: &str,
        body: Option<serde_json::Value>,
    ) -> Response {
        let mut builder = Request::builder().method(method).uri(path);
        builder = builder.header("authorization", format!("Bearer {token}"));
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        let request = builder
            .body(Body::from(
                body.map(|b| serde_json::to_string(&b).expect("valid json"))
                    .unwrap_or_default(),
            ))
            .expect("valid request");
        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("router responds")
    }
}

/// Reads the response body as JSON.
pub async fn body_json(response: Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("readable body");
    serde_json::from_slice(&body)
        .unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(&body).into_owned()))
}

/// Asserts that the response has the expected status code.
pub fn assert_status(response: &Response, expected: StatusCode) {
    assert_eq!(
        response.status(),
        expected,
        "unexpected status: {:?}",
        response.status()
    );
}

/// Builds an email address unique to the current test based on a seed.
pub fn test_email(seed: &str) -> String {
    format!("{}-{}@example.com", seed, uuid::Uuid::new_v4())
}
