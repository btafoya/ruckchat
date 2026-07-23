//! Custom emoji aggregate.

use ruckchat_common::{Error, Result, time::OffsetDateTime};
use ruckchat_id::{CustomEmojiId, FileId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};

/// A custom emoji uploaded to an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomEmoji {
    /// Internal emoji identifier.
    pub id: CustomEmojiId,
    /// Organization this emoji belongs to.
    pub organization_id: OrganizationId,
    /// Shortcode used to reference the emoji (without colons).
    pub shortcode: String,
    /// File storing the emoji image.
    pub file_id: FileId,
    /// User who created the emoji.
    pub created_by: UserId,
    /// Timestamp when the emoji was created.
    pub created_at: OffsetDateTime,
}

impl CustomEmoji {
    /// Creates a custom emoji after validating the shortcode.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Validation`] when the shortcode is empty or contains invalid characters.
    pub fn new(
        organization_id: OrganizationId,
        shortcode: impl Into<String>,
        file_id: FileId,
        created_by: UserId,
    ) -> Result<Self> {
        let shortcode = shortcode.into();

        if shortcode.is_empty() {
            return Err(Error::validation("emoji shortcode must not be empty"));
        }
        if !shortcode
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::validation(
                "emoji shortcode must contain only letters, numbers, hyphens, and underscores",
            ));
        }

        Ok(Self {
            id: CustomEmojiId::new(),
            organization_id,
            shortcode,
            file_id,
            created_by,
            created_at: OffsetDateTime::now_utc(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_valid_emoji() {
        let emoji = CustomEmoji::new(
            OrganizationId::new(),
            "partyparrot",
            FileId::new(),
            UserId::new(),
        )
        .expect("valid emoji");
        assert_eq!(emoji.shortcode, "partyparrot");
    }

    #[test]
    fn empty_shortcode_rejected() {
        assert!(CustomEmoji::new(OrganizationId::new(), "", FileId::new(), UserId::new()).is_err());
    }
}
