//! Static Web UI asset serving.
//!
//! Serves the browser-based React application from an embedded directory at
//! compile time or from a configured directory at runtime. Any path that does
//! not match a static file falls back to `index.html` so React Router can handle
//! client-side routes.

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};
use std::path::PathBuf;

use crate::state::AppState;

/// Web UI assets embedded from `web/dist` at compile time.
static EMBEDDED_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../web/dist");

/// Serves a static asset or falls back to `index.html`.
pub async fn serve_asset(
    State(state): State<AppState>,
    path: axum::extract::Path<String>,
) -> Response {
    let requested_path = &path.0;

    if state.web_config.enabled {
        if let Some(dir) = &state.web_config.path {
            return serve_from_dir(dir, requested_path).await;
        }
        return serve_embedded(requested_path);
    }

    StatusCode::NOT_FOUND.into_response()
}

/// Serves a file from a runtime directory.
async fn serve_from_dir(base_dir: &str, requested_path: &str) -> Response {
    let base = PathBuf::from(base_dir);
    let mut file_path = base.join(requested_path.trim_start_matches('/'));
    if requested_path.is_empty() || requested_path.ends_with('/') {
        file_path = file_path.join("index.html");
    }

    // Prevent directory traversal: the resolved path must stay under base_dir.
    let canonical_base = match tokio::fs::canonicalize(&base).await {
        Ok(path) => path,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let canonical_file = match tokio::fs::canonicalize(&file_path).await {
        Ok(path) => path,
        Err(_) => {
            // Fall back to index.html so React Router can handle the route.
            return serve_fallback_from_dir(&canonical_base).await;
        }
    };
    if !canonical_file.starts_with(&canonical_base) {
        return StatusCode::NOT_FOUND.into_response();
    }

    match tokio::fs::read(&canonical_file).await {
        Ok(contents) => asset_response(&canonical_file.to_string_lossy(), &contents),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Returns `index.html` from a runtime directory.
async fn serve_fallback_from_dir(base_dir: &std::path::Path) -> Response {
    let index_path = base_dir.join("index.html");
    match tokio::fs::read(&index_path).await {
        Ok(contents) => asset_response("index.html", &contents),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serves a file from the embedded `web/dist` directory.
fn serve_embedded(requested_path: &str) -> Response {
    let path = requested_path.trim_start_matches('/');
    if path.is_empty() {
        return serve_embedded_file("index.html");
    }

    if let Some(file) = EMBEDDED_ASSETS.get_file(path) {
        return asset_response(path, file.contents());
    }

    serve_embedded_file("index.html")
}

fn serve_embedded_file(path: &str) -> Response {
    match EMBEDDED_ASSETS.get_file(path) {
        Some(file) => asset_response(path, file.contents()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Builds an HTTP response for a static asset, setting a content type based on
/// the file extension.
fn asset_response(path: &str, contents: &[u8]) -> Response {
    let content_type = mime_type(path);
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        Vec::from(contents),
    )
        .into_response()
}

fn mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
        "application/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}
