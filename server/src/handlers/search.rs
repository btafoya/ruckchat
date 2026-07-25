//! Global search route handler.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::SearchResponse},
    services::dto::Pagination,
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use ruckchat_id::OrganizationId;
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for a search request.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    /// Raw search box text, including any `key:value` operators.
    pub q: String,
    /// Maximum results per content type.
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Result offset per content type.
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Searches messages, channels, people, and files within an organization.
pub async fn search(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(organization_id): Path<Uuid>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, Error> {
    let results = state
        .search
        .search(
            auth_user.id,
            OrganizationId::from_uuid(organization_id),
            &params.q,
            Pagination {
                limit: params.limit,
                offset: params.offset,
            },
        )
        .await?;
    Ok(Json(SearchResponse::from_results(results)))
}
