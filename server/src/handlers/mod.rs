//! HTTP route handlers and Axum router wiring.

pub mod auth;
pub mod channel;
pub mod direct_message;
pub mod dto;
pub mod error;
pub mod file;
pub mod message;
pub mod organization;
pub mod user;

pub use auth::AuthUser;

use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// Builds the full HTTP router for the RuckChat API.
pub fn router() -> Router<AppState> {
    Router::new()
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
        .route("/files", post(file::record))
        .route("/files/{file_id}", get(file::get_metadata))
        .route("/messages/{message_id}/attachments", post(file::attach))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
