//! File route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse},
    services::dto::{AttachFileRequest, RecordUploadRequest},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_id::{FileId, MessageId, OrganizationId};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for listing files in an organization.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFilesParams {
    /// Organization that owns the files.
    pub organization_id: OrganizationId,
}

/// Lists files in an organization.
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<ListFilesParams>,
) -> Result<Json<ListResponse<ruckchat_domain::File>>, Error> {
    let files = state
        .files
        .list_files_in_organization(auth_user.id, params.organization_id)
        .await?;
    Ok(Json(ListResponse::new(files)))
}

/// Records metadata for an uploaded file.
pub async fn record(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<RecordUploadRequest>,
) -> Result<impl IntoResponse, Error> {
    let file = state.files.record_upload(auth_user.id, request).await?;
    Ok((StatusCode::CREATED, Json(file)))
}

/// Returns metadata for a single file.
pub async fn get_metadata(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(file_id): Path<Uuid>,
) -> Result<Json<ruckchat_domain::File>, Error> {
    let file = state
        .files
        .get_file_metadata(FileId::from_uuid(file_id))
        .await?;
    Ok(Json(file))
}

/// Attaches a file to a message.
pub async fn attach(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
    Json(request): Json<AttachFileRequest>,
) -> Result<StatusCode, Error> {
    let attach_request = AttachFileRequest {
        message_id: MessageId::from_uuid(message_id),
        file_id: request.file_id,
    };
    state
        .files
        .attach_file_to_message(auth_user.id, attach_request)
        .await?;
    Ok(StatusCode::OK)
}
