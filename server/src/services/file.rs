//! File service.

use crate::services::dto::{AttachFileRequest, FileResponse, RecordUploadRequest};
use ruckchat_common::Error;
use ruckchat_domain::{
    File, FileRepository, MessageRepository, OrganizationMembershipRepository,
    OrganizationSettings, OrganizationSettingsRepository,
};
use ruckchat_id::{FileId, OrganizationId, UserId};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Dependencies required by [`FileService`].
#[derive(Clone)]
pub struct FileServiceDeps {
    /// File metadata repository.
    pub files: Arc<dyn FileRepository + Send + Sync>,
    /// Message repository.
    pub messages: Arc<dyn MessageRepository + Send + Sync>,
    /// Organization membership repository.
    pub memberships: Arc<dyn OrganizationMembershipRepository + Send + Sync>,
    /// Organization settings repository.
    pub settings: Arc<dyn OrganizationSettingsRepository + Send + Sync>,
}

/// File metadata, storage, and attachment operations.
#[derive(Clone)]
pub struct FileService {
    deps: FileServiceDeps,
    /// Directory where uploaded file bytes are stored on disk.
    storage_directory: PathBuf,
}

impl FileService {
    /// Creates the service from its dependencies and a storage directory.
    #[must_use]
    pub fn new(deps: FileServiceDeps, storage_directory: impl AsRef<Path>) -> Self {
        Self {
            deps,
            storage_directory: storage_directory.as_ref().to_path_buf(),
        }
    }

    /// Records metadata for an uploaded file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization
    /// member, [`Error::Validation`] for invalid metadata, and
    /// [`Error::Conflict`] when storage quota is exceeded.
    pub async fn record_upload(
        &self,
        caller_id: UserId,
        request: RecordUploadRequest,
    ) -> ruckchat_common::Result<FileResponse> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, request.organization_id)
            .await?;
        if membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        let settings = self
            .deps
            .settings
            .by_organization_id(request.organization_id)
            .await?
            .unwrap_or_else(|| OrganizationSettings::new(request.organization_id));

        if request.size_bytes > settings.max_file_size_bytes {
            return Err(Error::validation(format!(
                "file exceeds maximum size of {} bytes",
                settings.max_file_size_bytes
            )));
        }

        let file = File::new(
            request.organization_id,
            caller_id,
            request.file_name,
            request.mime_type,
            request.size_bytes,
            request.storage_path,
        )?;

        self.deps.files.create(&file).await?;

        Ok(FileResponse {
            id: file.id,
            file_name: file.file_name,
            mime_type: file.mime_type,
            size_bytes: file.size_bytes,
        })
    }

    /// Stores an uploaded file's bytes on disk and records its metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not an organization
    /// member, [`Error::Validation`] for invalid metadata, and
    /// [`Error::Internal`] when the file cannot be written to disk.
    pub async fn upload_file(
        &self,
        caller_id: UserId,
        request: crate::services::dto::UploadFileRequest,
        bytes: Vec<u8>,
    ) -> ruckchat_common::Result<FileResponse> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, request.organization_id)
            .await?;
        if membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        let settings = self
            .deps
            .settings
            .by_organization_id(request.organization_id)
            .await?
            .unwrap_or_else(|| OrganizationSettings::new(request.organization_id));

        let size_bytes =
            i64::try_from(bytes.len()).map_err(|_| Error::validation("file is too large"))?;
        if size_bytes > settings.max_file_size_bytes {
            return Err(Error::validation(format!(
                "file exceeds maximum size of {} bytes",
                settings.max_file_size_bytes
            )));
        }

        let mut file = File::new(
            request.organization_id,
            caller_id,
            request.file_name,
            request.mime_type,
            size_bytes,
            "pending",
        )?;

        let dir = self
            .storage_directory
            .join(file.organization_id.as_uuid().to_string());
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|err| Error::Internal(format!("failed to create file directory: {err}")))?;
        let storage_path = dir.join(file.id.as_uuid().to_string());
        tokio::fs::write(&storage_path, bytes)
            .await
            .map_err(|err| Error::Internal(format!("failed to write file: {err}")))?;

        file.storage_path = storage_path
            .to_str()
            .ok_or_else(|| Error::Internal("invalid file path".into()))?
            .to_string();

        self.deps.files.create(&file).await?;

        Ok(FileResponse {
            id: file.id,
            file_name: file.file_name,
            mime_type: file.mime_type,
            size_bytes: file.size_bytes,
        })
    }

    /// Loads file metadata by id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the file does not exist.
    pub async fn get_file_metadata(&self, file_id: FileId) -> ruckchat_common::Result<File> {
        self.deps
            .files
            .by_id(file_id)
            .await?
            .ok_or_else(|| Error::NotFound("file".into()))
    }

    /// Lists files in an organization. The caller must be a member.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Forbidden`] when the caller is not a member.
    pub async fn list_files_in_organization(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
    ) -> ruckchat_common::Result<Vec<File>> {
        let membership = self
            .deps
            .memberships
            .by_ids(caller_id, organization_id)
            .await?;
        if membership.is_none() {
            return Err(Error::Forbidden("must be an organization member".into()));
        }

        self.deps.files.list_by_organization(organization_id).await
    }

    /// Attaches a file to a message.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when the message or file does not exist and
    /// [`Error::Forbidden`] when the caller is neither the message author nor
    /// the file uploader.
    pub async fn attach_file_to_message(
        &self,
        caller_id: UserId,
        request: AttachFileRequest,
    ) -> ruckchat_common::Result<()> {
        let message = self
            .deps
            .messages
            .by_id(request.message_id)
            .await?
            .ok_or_else(|| Error::NotFound("message".into()))?;
        let file = self
            .deps
            .files
            .by_id(request.file_id)
            .await?
            .ok_or_else(|| Error::NotFound("file".into()))?;

        if message.author_id != caller_id && file.uploaded_by != caller_id {
            return Err(Error::Forbidden(
                "must be the message author or file uploader".into(),
            ));
        }

        self.deps
            .files
            .attach_to_message(request.message_id, request.file_id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::dto::{AttachFileRequest, RecordUploadRequest};
    use crate::testing::{
        MockFileRepository, MockMessageRepository, MockOrganizationMembershipRepository,
        MockOrganizationSettingsRepository,
    };
    use ruckchat_domain::{Channel, ConversationType, Message, OrganizationMembership, Role, User};
    use ruckchat_id::OrganizationId;
    use std::sync::Arc;

    fn service() -> FileService {
        let dir = std::env::temp_dir().join(format!("ruckchat-test-{}", uuid::Uuid::new_v4()));
        FileService::new(
            FileServiceDeps {
                files: Arc::new(MockFileRepository::new()),
                messages: Arc::new(MockMessageRepository::new()),
                memberships: Arc::new(MockOrganizationMembershipRepository::new()),
                settings: Arc::new(MockOrganizationSettingsRepository::new()),
            },
            dir,
        )
    }

    async fn seed_user_and_org(svc: &FileService) -> (UserId, OrganizationId) {
        let user = User::new("uploader@example.com", "Uploader", "hash").unwrap();
        let org_id = OrganizationId::new();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(user.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        (user.id, org_id)
    }

    #[tokio::test]
    async fn record_upload_creates_file() {
        let svc = service();
        let (user_id, org_id) = seed_user_and_org(&svc).await;
        let resp = svc
            .record_upload(
                user_id,
                RecordUploadRequest {
                    organization_id: org_id,
                    file_name: "report.pdf".into(),
                    mime_type: "application/pdf".into(),
                    size_bytes: 1024,
                    storage_path: "orgs/uuid/report.pdf".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.file_name, "report.pdf");
    }

    #[tokio::test]
    async fn record_upload_rejects_oversized_file() {
        let svc = service();
        let (user_id, org_id) = seed_user_and_org(&svc).await;
        let err = svc
            .record_upload(
                user_id,
                RecordUploadRequest {
                    organization_id: org_id,
                    file_name: "big.bin".into(),
                    mime_type: "application/octet-stream".into(),
                    size_bytes: 100 * 1024 * 1024,
                    storage_path: "path".into(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn attach_file_requires_author_or_uploader() {
        let svc = service();
        let (uploader_id, org_id) = seed_user_and_org(&svc).await;
        let author = User::new("author@example.com", "Author", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(author.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();

        let channel = Channel::new(org_id, "general", author.id, false).unwrap();
        let message = Message::new(
            channel.id.as_uuid(),
            ConversationType::Channel,
            author.id,
            "hello",
            None,
            vec![],
        )
        .unwrap();
        svc.deps.messages.create(&message).await.unwrap();

        let file_resp = svc
            .record_upload(
                uploader_id,
                RecordUploadRequest {
                    organization_id: org_id,
                    file_name: "report.pdf".into(),
                    mime_type: "application/pdf".into(),
                    size_bytes: 1024,
                    storage_path: "path".into(),
                },
            )
            .await
            .unwrap();

        svc.attach_file_to_message(
            author.id,
            AttachFileRequest {
                message_id: message.id,
                file_id: file_resp.id,
            },
        )
        .await
        .unwrap();

        let outsider = User::new("outsider@example.com", "Outsider", "hash").unwrap();
        svc.deps
            .memberships
            .create(&OrganizationMembership::new(outsider.id, org_id, Role::Member).unwrap())
            .await
            .unwrap();
        let err = svc
            .attach_file_to_message(
                outsider.id,
                AttachFileRequest {
                    message_id: message.id,
                    file_id: file_resp.id,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }
}
