//! Cross-content-type search: messages, channels, people, and files.
//!
//! Message search reuses the existing Postgres full-text search already
//! backing the MCP `search_messages` tool. Channels, people, and files reuse
//! the "load the visible set, filter in-memory" pattern already established
//! by [`crate::services::organization::OrganizationService::search_members`]
//! for @mention autocomplete, since those tables are small per-organization
//! and don't need stemming or ranking.

use crate::services::channel::ChannelService;
use crate::services::dto::Pagination;
use crate::services::file::FileService;
use crate::services::message::MessageService;
use crate::services::organization::OrganizationService;
use ruckchat_domain::{Channel, File, Message, MessageReadRepository, User};
use ruckchat_id::{MessageId, OrganizationId, UserId};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use time::Date;

/// A search query parsed into free text and Gmail-style operators.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchQuery {
    /// Terms remaining after stripping recognized operators.
    pub free_text: String,
    /// `from:` operator value, matched against the author's display name.
    pub from: Option<String>,
    /// `in:` operator value, matched against channel name.
    pub in_: Option<String>,
    /// `has:attachment` operator.
    pub has_attachment: bool,
    /// `after:` operator value.
    pub after: Option<Date>,
    /// `before:` operator value.
    pub before: Option<Date>,
    /// `is:unread` operator.
    pub is_unread: bool,
}

/// Parses a raw search box query into free text and recognized operators.
/// Unrecognized `key:value` tokens (including malformed dates) are treated
/// as free text.
#[must_use]
pub fn parse_query(raw: &str) -> SearchQuery {
    let mut query = SearchQuery::default();
    let mut free_terms = Vec::new();

    for token in raw.split_whitespace() {
        if let Some(value) = token.strip_prefix("from:") {
            query.from = Some(value.to_string());
        } else if let Some(value) = token.strip_prefix("in:") {
            query.in_ = Some(value.to_string());
        } else if let Some(value) = token.strip_prefix("has:") {
            if value.eq_ignore_ascii_case("attachment") {
                query.has_attachment = true;
            } else {
                free_terms.push(token);
            }
        } else if let Some(value) = token.strip_prefix("before:") {
            match parse_date(value) {
                Some(date) => query.before = Some(date),
                None => free_terms.push(token),
            }
        } else if let Some(value) = token.strip_prefix("after:") {
            match parse_date(value) {
                Some(date) => query.after = Some(date),
                None => free_terms.push(token),
            }
        } else if let Some(value) = token.strip_prefix("is:") {
            if value.eq_ignore_ascii_case("unread") {
                query.is_unread = true;
            } else {
                free_terms.push(token);
            }
        } else {
            free_terms.push(token);
        }
    }

    query.free_text = free_terms.join(" ");
    query
}

/// Parses a `YYYY-MM-DD` date, returning `None` for any other shape.
fn parse_date(s: &str) -> Option<Date> {
    let mut parts = s.splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let month = time::Month::try_from(month).ok()?;
    Date::from_calendar_date(year, month, day).ok()
}

/// Search results grouped by content type.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchResults {
    /// Matching messages.
    pub messages: Vec<Message>,
    /// Matching channels.
    pub channels: Vec<Channel>,
    /// Matching organization members.
    pub people: Vec<User>,
    /// Matching files.
    pub files: Vec<File>,
}

/// Dependencies required by [`SearchService`].
#[derive(Clone)]
pub struct SearchServiceDeps {
    /// Message service, reused for its existing full-text search.
    pub messages: MessageService,
    /// Channel service, reused for its visibility-filtered channel listing.
    pub channels: ChannelService,
    /// Organization service, reused for member search.
    pub organizations: OrganizationService,
    /// File service, reused for org file listing and attachment lookups.
    pub files: FileService,
    /// Message read-state repository, used by the `is:unread` operator.
    pub reads: Arc<dyn MessageReadRepository + Send + Sync>,
}

/// Cross-content-type search operations.
#[derive(Clone)]
pub struct SearchService {
    deps: SearchServiceDeps,
}

impl SearchService {
    /// Creates the service from its dependencies.
    #[must_use]
    pub fn new(deps: SearchServiceDeps) -> Self {
        Self { deps }
    }

    /// Searches messages, channels, people, and files visible to the caller
    /// within an organization.
    ///
    /// # Errors
    ///
    /// Returns [`ruckchat_common::Error::Forbidden`] when the caller is not an
    /// organization member.
    pub async fn search(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        raw_query: &str,
        pagination: Pagination,
    ) -> ruckchat_common::Result<SearchResults> {
        let query = parse_query(raw_query);
        let pagination = pagination.normalized();

        let messages = self
            .search_messages(caller_id, organization_id, &query, pagination)
            .await?;
        let channels = self
            .search_channels(caller_id, organization_id, &query.free_text)
            .await?;
        let people = self
            .deps
            .organizations
            .search_members(caller_id, organization_id, &query.free_text)
            .await?;
        let files = self
            .search_files(caller_id, organization_id, &query.free_text)
            .await?;

        Ok(SearchResults {
            messages,
            channels,
            people,
            files,
        })
    }

    async fn search_messages(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        query: &SearchQuery,
        pagination: Pagination,
    ) -> ruckchat_common::Result<Vec<Message>> {
        let mut messages = self
            .deps
            .messages
            .search_messages(caller_id, organization_id, &query.free_text, pagination)
            .await?;

        if let Some(from) = &query.from {
            let from_lower = from.to_lowercase();
            messages.retain(|m| {
                m.author_display_name
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&from_lower)
            });
        }

        if let Some(in_) = &query.in_ {
            let channels = self
                .deps
                .channels
                .list_channels_in_organization(caller_id, organization_id)
                .await?;
            let in_lower = in_.to_lowercase();
            let matching_ids: HashSet<uuid::Uuid> = channels
                .into_iter()
                .filter(|c| c.name.to_lowercase().contains(&in_lower))
                .map(|c| c.id.as_uuid())
                .collect();
            messages.retain(|m| matching_ids.contains(&m.conversation_id));
        }

        if query.has_attachment {
            let ids: Vec<MessageId> = messages.iter().map(|m| m.id).collect();
            let with_attachments = self.deps.files.message_ids_with_attachments(&ids).await?;
            messages.retain(|m| with_attachments.contains(&m.id));
        }

        if let Some(after) = query.after {
            messages.retain(|m| m.created_at.date() >= after);
        }
        if let Some(before) = query.before {
            messages.retain(|m| m.created_at.date() <= before);
        }

        if query.is_unread {
            let ids: Vec<MessageId> = messages.iter().map(|m| m.id).collect();
            let unread = self.deps.reads.unread_message_ids(caller_id, &ids).await?;
            messages.retain(|m| m.author_id != caller_id && unread.contains(&m.id));
        }

        Ok(messages)
    }

    async fn search_channels(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        free_text: &str,
    ) -> ruckchat_common::Result<Vec<Channel>> {
        let channels = self
            .deps
            .channels
            .list_channels_in_organization(caller_id, organization_id)
            .await?;
        if free_text.is_empty() {
            return Ok(channels);
        }
        let lower = free_text.to_lowercase();
        Ok(channels
            .into_iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&lower)
                    || c.topic
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&lower)
                    || c.purpose
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&lower)
            })
            .collect())
    }

    async fn search_files(
        &self,
        caller_id: UserId,
        organization_id: OrganizationId,
        free_text: &str,
    ) -> ruckchat_common::Result<Vec<File>> {
        let files = self
            .deps
            .files
            .list_files_in_organization(caller_id, organization_id)
            .await?;
        if free_text.is_empty() {
            return Ok(files);
        }
        let lower = free_text.to_lowercase();
        Ok(files
            .into_iter()
            .filter(|f| f.file_name.to_lowercase().contains(&lower))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_free_text_only() {
        let query = parse_query("deploy failed");
        assert_eq!(query.free_text, "deploy failed");
        assert_eq!(query.from, None);
    }

    #[test]
    fn parses_from_and_in_operators() {
        let query = parse_query("deploy from:alice in:general");
        assert_eq!(query.free_text, "deploy");
        assert_eq!(query.from.as_deref(), Some("alice"));
        assert_eq!(query.in_.as_deref(), Some("general"));
    }

    #[test]
    fn parses_has_attachment() {
        let query = parse_query("report has:attachment");
        assert_eq!(query.free_text, "report");
        assert!(query.has_attachment);
    }

    #[test]
    fn unknown_has_value_is_free_text() {
        let query = parse_query("has:wings");
        assert_eq!(query.free_text, "has:wings");
        assert!(!query.has_attachment);
    }

    #[test]
    fn parses_is_unread() {
        let query = parse_query("is:unread standup");
        assert_eq!(query.free_text, "standup");
        assert!(query.is_unread);
    }

    #[test]
    fn parses_date_range() {
        let query = parse_query("after:2026-01-01 before:2026-02-01 notes");
        assert_eq!(query.free_text, "notes");
        assert_eq!(
            query.after,
            Some(Date::from_calendar_date(2026, time::Month::January, 1).unwrap())
        );
        assert_eq!(
            query.before,
            Some(Date::from_calendar_date(2026, time::Month::February, 1).unwrap())
        );
    }

    #[test]
    fn malformed_date_falls_back_to_free_text() {
        let query = parse_query("after:not-a-date");
        assert_eq!(query.free_text, "after:not-a-date");
        assert_eq!(query.after, None);
    }

    #[test]
    fn empty_query_has_empty_free_text() {
        let query = parse_query("   ");
        assert_eq!(query.free_text, "");
    }
}
