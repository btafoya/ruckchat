//! File attachment aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{FileId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// Metadata for an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    /// Internal file identifier.
    pub id: FileId,
    /// Organization the file belongs to.
    pub organization_id: OrganizationId,
    /// User who uploaded the file.
    pub uploaded_by: UserId,
    /// Original file name.
    pub file_name: String,
    /// MIME type.
    pub mime_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
    /// Storage backend path or key.
    pub storage_path: String,
    /// Optional thumbnail path or key.
    pub thumbnail_path: Option<String>,
    /// Timestamp when the file was uploaded.
    pub created_at: OffsetDateTime,
}

impl File {
    /// Creates a new file record after validating required fields.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when any required field is empty or the
    /// size is not positive.
    pub fn new(
        organization_id: OrganizationId,
        uploaded_by: UserId,
        file_name: impl Into<String>,
        mime_type: impl Into<String>,
        size_bytes: i64,
        storage_path: impl Into<String>,
    ) -> Result<Self> {
        let file_name = file_name.into();
        let mime_type = mime_type.into();
        let storage_path = storage_path.into();

        if file_name.is_empty() {
            return Err(Error::validation("file name must not be empty"));
        }
        if mime_type.is_empty() {
            return Err(Error::validation("mime type must not be empty"));
        }
        if storage_path.is_empty() {
            return Err(Error::validation("storage path must not be empty"));
        }
        if size_bytes <= 0 {
            return Err(Error::validation("file size must be positive"));
        }

        Ok(Self {
            id: FileId::new(),
            organization_id,
            uploaded_by,
            file_name,
            mime_type,
            size_bytes,
            storage_path,
            thumbnail_path: None,
            created_at: OffsetDateTime::now_utc(),
        })
    }

    /// Sets the thumbnail path.
    pub fn set_thumbnail_path(&mut self, thumbnail_path: Option<impl Into<String>>) {
        self.thumbnail_path = thumbnail_path.map(Into::into);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_file() {
        let file = File::new(
            OrganizationId::new(),
            UserId::new(),
            "report.pdf",
            "application/pdf",
            1024,
            "orgs/uuid/report.pdf",
        )
        .expect("valid file");
        assert_eq!(file.file_name, "report.pdf");
        assert_eq!(file.size_bytes, 1024);
    }

    #[test]
    fn invalid_file_rejected() {
        let org_id = OrganizationId::new();
        let user_id = UserId::new();
        assert!(File::new(org_id, user_id, "", "application/pdf", 1024, "path").is_err());
        assert!(File::new(org_id, user_id, "file", "", 1024, "path").is_err());
        assert!(File::new(org_id, user_id, "file", "application/pdf", 0, "path").is_err());
        assert!(File::new(org_id, user_id, "file", "application/pdf", 1024, "").is_err());
    }
}
