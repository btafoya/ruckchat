//! File route handlers.

use crate::{
    Error,
    handlers::{auth::AuthUser, dto::ListResponse},
    services::dto::{AttachFileRequest, RecordUploadRequest, UploadFileRequest},
    state::AppState,
};
use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use ruckchat_common::Error as DomainError;
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

/// Accepts a multipart file upload and stores the file bytes on disk.
pub async fn upload(
    State(state): State<AppState>,
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let mut organization_id: Option<OrganizationId> = None;
    let mut file_name: Option<String> = None;
    let mut mime_type: String = "application/octet-stream".into();
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(map_multipart_err)? {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "organization_id" => {
                let value = field.text().await.map_err(map_multipart_err)?;
                organization_id =
                    Some(OrganizationId::parse_str(&value).map_err(|err| {
                        DomainError::validation(format!("organization_id: {err}"))
                    })?);
            }
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                if let Some(content_type) = field.content_type() {
                    mime_type = content_type.to_string();
                }
                bytes = Some(field.bytes().await.map_err(map_multipart_err)?.to_vec());
            }
            _ => {
                // Ignore unknown fields.
            }
        }
    }

    let Some(organization_id) = organization_id else {
        return Err(DomainError::validation("organization_id is required").into());
    };
    let Some(file_name) = file_name else {
        return Err(DomainError::validation("file is required").into());
    };
    let Some(bytes) = bytes else {
        return Err(DomainError::validation("file is required").into());
    };

    let request = UploadFileRequest {
        organization_id,
        file_name,
        mime_type,
    };
    let file = state
        .files
        .upload_file(auth_user.id, request, bytes)
        .await?;
    Ok((StatusCode::CREATED, Json(file)))
}

fn map_multipart_err(err: axum::extract::multipart::MultipartError) -> Error {
    DomainError::validation(format!("multipart error: {err}")).into()
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
