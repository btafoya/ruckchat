//! Axum handler for the Streamable HTTP MCP endpoint.
//!
//! The handler authenticates the request via the existing [`AuthUser`]
//! extractor, injects the caller's [`UserId`] into the request extensions, and
//! dispatches to an [`rmcp`] [`StreamableHttpService`].

use crate::{
    handlers::auth::AuthUser, mcp::server::RuckChatMcpServer, services::McpService, state::AppState,
};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, session::local::LocalSessionManager, tower::StreamableHttpService,
};
use std::sync::Arc;

/// HTTP service wrapper that runs the `rmcp` Streamable HTTP transport.
#[derive(Clone)]
pub struct McpHttpService {
    inner: Arc<StreamableHttpService<RuckChatMcpServer, LocalSessionManager>>,
}

impl McpHttpService {
    /// Creates the MCP HTTP service from an [`McpService`].
    #[must_use]
    pub fn new(mcp: McpService) -> Self {
        let session_manager = Arc::new(LocalSessionManager::default());
        let config = StreamableHttpServerConfig::default().with_stateful_mode(true);
        let service = StreamableHttpService::new(
            move || Ok::<RuckChatMcpServer, std::io::Error>(RuckChatMcpServer::new(mcp.clone())),
            session_manager,
            config,
        );
        Self {
            inner: Arc::new(service),
        }
    }

    /// Handles an authenticated HTTP request.
    pub async fn handle(&self, request: Request<Body>) -> Response<Body> {
        let response = self.inner.handle(request).await;
        response.map(Body::new)
    }
}

/// Axum route handler for `/mcp/v1/sse`.
pub async fn mcp_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    mut request: Request<Body>,
) -> Response<Body> {
    if !state.mcp_enabled {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .expect("valid response");
    }

    request.extensions_mut().insert(auth_user.id);
    state.mcp_http.handle(request).await
}
