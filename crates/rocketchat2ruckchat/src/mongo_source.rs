//! Reads RocketChat data directly from a restored MongoDB dump.
//!
//! The source is always a locally restored `mongorestore` target, never a
//! live production `mongod` (see `docs/DESIGN-RocketChat-Migration.md`).
//! File bytes are read directly from Mongo's GridFS chunk/file collections
//! rather than through the driver's GridFS bucket helper, since that API's
//! bucket-naming type isn't part of the crate's public surface; the GridFS
//! wire format itself is stable and simple enough to read directly.

use std::collections::HashMap;

use futures_util::TryStreamExt;
use mongodb::bson::{Binary, doc};
use mongodb::{Client, Collection, Database};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::error::Result;

/// Connection to a restored RocketChat MongoDB dump.
pub struct MongoSource {
    db: Database,
}

impl MongoSource {
    /// Connects to the given Mongo URI and selects the database.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the connection cannot be established.
    pub async fn connect(uri: &str, database: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        Ok(Self {
            db: client.database(database),
        })
    }

    /// Reads the applied schema-migration version from the `migrations`
    /// collection's `control` document, for compatibility warnings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn migration_version(&self) -> Result<Option<i64>> {
        #[derive(Deserialize)]
        struct MigrationControl {
            version: i64,
        }

        let control: Option<MigrationControl> = self
            .db
            .collection("migrations")
            .find_one(doc! { "_id": "control" })
            .await?;
        Ok(control.map(|c| c.version))
    }

    /// Counts documents in a named collection, for dry-run inventory reporting.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn count(&self, collection: &str) -> Result<u64> {
        let count = self
            .db
            .collection::<mongodb::bson::Document>(collection)
            .count_documents(doc! {})
            .await?;
        Ok(count)
    }

    /// Lists every user document.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn list_users(&self) -> Result<Vec<RocketUser>> {
        let cursor = self
            .db
            .collection::<RocketUser>("users")
            .find(doc! {})
            .await?;
        Ok(cursor.try_collect().await?)
    }

    /// Lists every room document, excluding Omnichannel/Livechat rooms
    /// (`t: "l"`), which are out of scope.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn list_rooms(&self) -> Result<Vec<RocketRoom>> {
        let cursor = self
            .db
            .collection::<RocketRoom>("rocketchat_room")
            .find(doc! { "t": { "$ne": "l" } })
            .await?;
        Ok(cursor.try_collect().await?)
    }

    /// Lists every subscription document (the per-user, per-room membership
    /// record RocketChat uses for channels, groups, and direct messages
    /// alike).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn list_subscriptions(&self) -> Result<Vec<RocketSubscription>> {
        let cursor = self
            .db
            .collection::<RocketSubscription>("rocketchat_subscription")
            .find(doc! {})
            .await?;
        Ok(cursor.try_collect().await?)
    }

    /// Lists every message in a room, ordered by timestamp then id to match
    /// the `rid_1_ts_1__updatedAt_1` index RocketChat already maintains.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn list_messages_for_room(&self, room_id: &str) -> Result<Vec<RocketMessage>> {
        let cursor = self
            .db
            .collection::<RocketMessage>("rocketchat_message")
            .find(doc! { "rid": room_id })
            .sort(doc! { "ts": 1, "_id": 1 })
            .await?;
        Ok(cursor.try_collect().await?)
    }

    /// Lists every custom emoji document.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn list_custom_emoji(&self) -> Result<Vec<RocketEmoji>> {
        let cursor = self
            .db
            .collection::<RocketEmoji>("rocketchat_custom_emoji")
            .find(doc! {})
            .await?;
        Ok(cursor.try_collect().await?)
    }

    /// Looks up an upload's metadata document by id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn find_upload(&self, file_id: &str) -> Result<Option<RocketUpload>> {
        let upload = self
            .db
            .collection::<RocketUpload>("rocketchat_uploads")
            .find_one(doc! { "_id": file_id })
            .await?;
        Ok(upload)
    }

    /// Reads a file's bytes directly out of its GridFS bucket, reassembling
    /// chunks in order. Returns `None` when the upload's `store` field isn't
    /// GridFS-backed, or when no chunks are found for the id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mongo`] when the query fails.
    pub async fn download_gridfs_bytes(
        &self,
        store: &str,
        file_id: &str,
    ) -> Result<Option<Vec<u8>>> {
        let Some(bucket) = gridfs_bucket_prefix(store) else {
            return Ok(None);
        };

        let chunks: Collection<GridFsChunkDoc> = self.db.collection(&format!("{bucket}.chunks"));
        let mut cursor = chunks
            .find(doc! { "files_id": file_id })
            .sort(doc! { "n": 1 })
            .await?;

        let mut bytes = Vec::new();
        let mut found = false;
        while let Some(chunk) = cursor.try_next().await? {
            found = true;
            bytes.extend_from_slice(&chunk.data.bytes);
        }
        Ok(found.then_some(bytes))
    }
}

/// Maps a RocketChat `store` field (e.g. `"GridFS:Uploads"`) to the Mongo
/// GridFS bucket prefix RocketChat's own store adapters use
/// (`"rocketchat_uploads"`, matching the real `rocketchat_uploads.files` /
/// `.chunks` collections). Returns `None` for non-GridFS stores (S3,
/// GoogleStorage, FileSystem), which are out of scope for this v1.
fn gridfs_bucket_prefix(store: &str) -> Option<String> {
    let suffix = store.strip_prefix("GridFS:")?;
    let mut chars = suffix.chars();
    let first = chars.next()?.to_ascii_lowercase();
    Some(format!("rocketchat_{first}{}", chars.as_str()))
}

/// A model for a document in a GridFS bucket's chunks collection. `n` (chunk
/// sequence number) is required for the sort in the query that fetches these
/// but isn't read again afterward, since the cursor already yields them in
/// order.
#[derive(Debug, Deserialize)]
struct GridFsChunkDoc {
    #[allow(dead_code)]
    n: i32,
    data: Binary,
}

/// Converts a BSON datetime to an [`OffsetDateTime`], defaulting to `fallback`
/// when absent.
#[must_use]
pub fn bson_datetime_or(
    value: Option<mongodb::bson::DateTime>,
    fallback: OffsetDateTime,
) -> OffsetDateTime {
    value
        .map(|dt| OffsetDateTime::UNIX_EPOCH + time::Duration::milliseconds(dt.timestamp_millis()))
        .unwrap_or(fallback)
}

/// Lightweight user reference embedded in rooms, messages, and subscriptions.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketUserRef {
    /// User identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Username.
    pub username: Option<String>,
}

/// A single email entry on a user document.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketEmail {
    /// Email address.
    pub address: String,
}

/// RocketChat user record.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketUser {
    /// Internal RocketChat identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Unique username.
    pub username: String,
    /// Display name.
    pub name: Option<String>,
    /// Whether the account is active.
    #[serde(default = "default_true")]
    pub active: bool,
    /// Email addresses.
    #[serde(default)]
    pub emails: Vec<RocketEmail>,
    /// Timestamp when the user was created.
    #[serde(rename = "createdAt")]
    pub created_at: Option<mongodb::bson::DateTime>,
}

fn default_true() -> bool {
    true
}

/// RocketChat room record (channel, private group, direct message, or
/// discussion). Livechat rooms (`t: "l"`) are filtered out by the query.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketRoom {
    /// Internal RocketChat identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Room type: `c` (public), `p` (private), `d` (direct message).
    #[serde(rename = "t")]
    pub room_type: String,
    /// Room name (channels/groups only).
    pub name: Option<String>,
    /// Friendly display name.
    pub fname: Option<String>,
    /// Topic.
    pub topic: Option<String>,
    /// Description/purpose.
    pub description: Option<String>,
    /// Whether the room is archived.
    #[serde(default)]
    pub archived: bool,
    /// Room creator.
    pub u: Option<RocketUserRef>,
    /// Timestamp when the room was created.
    pub ts: Option<mongodb::bson::DateTime>,
    /// Parent room identifier, set when this room is a discussion.
    pub prid: Option<String>,
}

/// RocketChat subscription record: one row per user per room, used to build
/// channel memberships and direct-message participant lists alike.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketSubscription {
    /// Room identifier.
    pub rid: String,
    /// Subscribed user.
    pub u: RocketUserRef,
}

/// A single reaction entry, keyed by emoji shortcode on the message document.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RocketReactionEntry {
    /// Usernames who added this reaction.
    #[serde(default)]
    pub usernames: Vec<String>,
}

/// A file reference embedded in a message (`file` or an entry in `files`).
#[derive(Debug, Clone, Deserialize)]
pub struct RocketMessageFileRef {
    /// Upload identifier, joined against `rocketchat_uploads`.
    #[serde(rename = "_id")]
    pub id: String,
}

/// RocketChat message record.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketMessage {
    /// Message identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Room identifier.
    pub rid: String,
    /// System-message discriminator. `None`/absent means a real chat message;
    /// any other value (`au`, `uj`, `r`, `message_pinned`, ...) is a system
    /// event and is skipped entirely (see the design doc's non-goals).
    #[serde(rename = "t")]
    pub system_type: Option<String>,
    /// Author.
    pub u: RocketUserRef,
    /// Message text.
    #[serde(default)]
    pub msg: String,
    /// Timestamp when the message was created.
    pub ts: Option<mongodb::bson::DateTime>,
    /// Last-edit timestamp.
    #[serde(rename = "editedAt")]
    pub edited_at: Option<mongodb::bson::DateTime>,
    /// Thread parent message identifier.
    pub tmid: Option<String>,
    /// Reactions, keyed by emoji shortcode.
    #[serde(default)]
    pub reactions: HashMap<String, RocketReactionEntry>,
    /// Single legacy file reference.
    pub file: Option<RocketMessageFileRef>,
    /// Modern multi-file references.
    #[serde(default)]
    pub files: Vec<RocketMessageFileRef>,
}

/// RocketChat custom emoji record.
#[derive(Debug, Clone, Deserialize)]
pub struct RocketEmoji {
    /// Emoji identifier.
    #[serde(rename = "_id")]
    pub id: String,
    /// Shortcode without surrounding colons.
    pub name: String,
    /// File extension.
    pub extension: Option<String>,
}

/// RocketChat upload metadata record (`rocketchat_uploads` collection).
#[derive(Debug, Clone, Deserialize)]
pub struct RocketUpload {
    /// Upload identifier, matching the GridFS file id.
    #[serde(rename = "_id")]
    pub id: String,
    /// Original file name.
    pub name: Option<String>,
    /// MIME type.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// Size in bytes.
    pub size: Option<i64>,
    /// Storage adapter identifier, e.g. `"GridFS:Uploads"`.
    pub store: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gridfs_bucket_prefix_maps_known_stores() {
        assert_eq!(
            gridfs_bucket_prefix("GridFS:Uploads").as_deref(),
            Some("rocketchat_uploads")
        );
        assert_eq!(
            gridfs_bucket_prefix("GridFS:Avatars").as_deref(),
            Some("rocketchat_avatars")
        );
        assert_eq!(
            gridfs_bucket_prefix("GridFS:UserDataFiles").as_deref(),
            Some("rocketchat_userDataFiles")
        );
    }

    #[test]
    fn gridfs_bucket_prefix_rejects_non_gridfs_stores() {
        assert_eq!(gridfs_bucket_prefix("AmazonS3:Uploads"), None);
        assert_eq!(gridfs_bucket_prefix("FileSystem:Uploads"), None);
    }

    #[test]
    fn bson_datetime_or_uses_fallback_when_absent() {
        let fallback = OffsetDateTime::UNIX_EPOCH;
        assert_eq!(bson_datetime_or(None, fallback), fallback);
    }
}
