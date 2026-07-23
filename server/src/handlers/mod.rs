//! HTTP route handlers and Axum router wiring.

pub mod auth;
pub mod channel;
pub mod direct_message;
pub mod dto;
pub mod error;
pub mod file;
pub mod message;
pub mod organization;
pub mod plugins;
pub mod reaction;
pub mod user;
pub mod web_assets;
pub mod web_push;

pub use auth::AuthUser;

use crate::{mcp::mcp_handler, state::AppState, websocket::websocket_handler};
use axum::{
    Router,
    http::header::{AUTHORIZATION, CONTENT_TYPE},
    routing::{any, delete, get, patch, post},
};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

/// Builds the full HTTP router for the RuckChat API.
pub fn router(web_config: &ruckchat_config::WebConfig, base_url: &str) -> Router<AppState> {
    Router::new()
        .route("/websocket", get(websocket_handler))
        .route("/mcp/v1/sse", any(mcp_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/users/me", get(user::get_profile))
        .route("/users/me", patch(user::update_profile))
        .route("/organizations", get(organization::list))
        .route("/organizations", post(organization::create))
        .route(
            "/organizations/{organization_id}/members",
            post(organization::invite_member),
        )
        .route(
            "/organizations/{organization_id}/members",
            patch(organization::change_role),
        )
        .route(
            "/organizations/{organization_id}/members",
            delete(organization::remove_member),
        )
        .route(
            "/organizations/{organization_id}/channels",
            get(channel::list_in_organization),
        )
        .route(
            "/organizations/{organization_id}/channels",
            post(channel::create),
        )
        .route("/channels/{channel_id}", get(channel::get))
        .route("/channels/{channel_id}", patch(channel::update))
        .route("/channels/{channel_id}", delete(channel::archive))
        .route("/channels/{channel_id}/members", post(channel::add_member))
        .route(
            "/channels/{channel_id}/members",
            delete(channel::remove_member),
        )
        .route(
            "/channels/{channel_id}/messages",
            get(message::list_history),
        )
        .route("/channels/{channel_id}/messages", post(message::post))
        .route("/messages/{message_id}", patch(message::edit))
        .route("/messages/{message_id}", delete(message::delete))
        .route("/messages/{message_id}/replies", get(message::list_replies))
        .route("/messages/{message_id}/reactions", post(reaction::add))
        .route(
            "/messages/{message_id}/reactions/{emoji}",
            delete(reaction::remove),
        )
        .route("/direct_messages", get(direct_message::list_conversations))
        .route("/direct_messages", post(direct_message::start))
        .route(
            "/direct_messages/{conversation_id}/messages",
            get(direct_message::list_messages),
        )
        .route(
            "/direct_messages/{conversation_id}/messages",
            post(direct_message::post_message),
        )
        .route("/files", get(file::list))
        .route("/files", post(file::upload))
        .route("/files/record", post(file::record))
        .route("/files/{file_id}", get(file::get_metadata))
        .route("/messages/{message_id}/attachments", post(file::attach))
        .route(
            "/plugins/{plugin}/commands/{command}",
            post(plugins::invoke_command),
        )
        .route("/web-push/vapid-key", get(web_push::vapid_key))
        .route("/web-push/subscribe", post(web_push::subscribe))
        .route("/web-push/unsubscribe", post(web_push::unsubscribe))
        .route("/", get(web_assets::serve_root))
        .route("/{*path}", get(web_assets::serve_asset))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer(web_config, base_url))
}

/// Builds a CORS layer that allows credentialed requests from configured
/// origins.
fn cors_layer(web_config: &ruckchat_config::WebConfig, base_url: &str) -> CorsLayer {
    let mut origins = web_config.allowed_origins.clone();
    if origins.is_empty()
        && let Ok(url) = url::Url::parse(base_url)
    {
        origins.push(url.origin().ascii_serialization());
    }

    let allow_origin = if origins.is_empty() {
        AllowOrigin::any()
    } else {
        let header_values: Vec<axum::http::HeaderValue> = origins
            .into_iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        AllowOrigin::list(header_values)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_credentials(true)
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ]))
        .allow_headers(AllowHeaders::list([
            CONTENT_TYPE,
            AUTHORIZATION,
            axum::http::header::ACCEPT,
        ]))
}
