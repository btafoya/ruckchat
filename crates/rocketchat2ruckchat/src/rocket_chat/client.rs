//! RocketChat REST API client.

use std::collections::HashMap;

use reqwest::{Client, StatusCode};
use tracing::{debug, warn};

use crate::config::RocketAuthConfig;
use crate::error::{Error, Result};
use crate::rocket_chat::auth::{RocketSession, authenticate};
use crate::rocket_chat::models::{
    RocketAttachment, RocketEmoji, RocketListResponse, RocketMessage, RocketRole, RocketRoom,
    RocketTeam, RocketUser,
};

const PAGE_SIZE: u64 = 100;

/// Client for the RocketChat REST API.
#[derive(Debug, Clone)]
pub struct RocketChatClient {
    http: Client,
    session: RocketSession,
}

impl RocketChatClient {
    /// Authenticates and creates a new client.
    pub async fn new(base_url: &str, auth: &RocketAuthConfig) -> Result<Self> {
        let http = Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|e| Error::Internal(e.to_string()))?;
        let session = authenticate(&http, base_url, auth).await?;
        Ok(Self { http, session })
    }

    /// Lists all users.
    pub async fn list_users(&self) -> Result<Vec<RocketUser>> {
        self.paginate_typed("/api/v1/users.listAll", "users").await
    }

    /// Lists public channels.
    pub async fn list_channels(&self) -> Result<Vec<RocketRoom>> {
        self.paginate_typed("/api/v1/channels.list", "channels")
            .await
    }

    /// Lists private groups.
    pub async fn list_groups(&self) -> Result<Vec<RocketRoom>> {
        self.paginate_typed("/api/v1/groups.list", "groups").await
    }

    /// Lists direct message rooms.
    pub async fn list_ims(&self) -> Result<Vec<RocketRoom>> {
        self.paginate_typed("/api/v1/im.list", "ims").await
    }

    /// Lists teams.
    pub async fn list_teams(&self) -> Result<Vec<RocketTeam>> {
        self.paginate_typed("/api/v1/teams.list", "teams").await
    }

    /// Lists custom emoji.
    pub async fn list_emoji(&self) -> Result<Vec<RocketEmoji>> {
        self.paginate_typed("/api/v1/emoji-custom.list", "emoji")
            .await
    }

    /// Lists roles.
    pub async fn list_roles(&self) -> Result<Vec<RocketRole>> {
        self.paginate_typed("/api/v1/roles.list", "roles").await
    }

    /// Lists messages in a channel room.
    pub async fn channel_history(&self, room_id: &str) -> Result<Vec<RocketMessage>> {
        self.paginate_messages("/api/v1/channels.history", room_id)
            .await
    }

    /// Lists messages in a private group.
    pub async fn group_history(&self, room_id: &str) -> Result<Vec<RocketMessage>> {
        self.paginate_messages("/api/v1/groups.history", room_id)
            .await
    }

    /// Lists messages in a direct-message room.
    pub async fn im_history(&self, room_id: &str) -> Result<Vec<RocketMessage>> {
        self.paginate_messages("/api/v1/im.history", room_id).await
    }

    /// Downloads a file from RocketChat.
    pub async fn download_file(&self, url: &str) -> Result<(reqwest::Response, Option<String>)> {
        let full_url = self.session.url(url);
        let response = self.http.get(&full_url).send().await?;
        if !response.status().is_success() {
            return Err(Error::rocketchat(
                Some(response.status()),
                format!("failed to download {full_url}"),
            ));
        }
        let filename = response
            .headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_filename)
            .or_else(|| {
                url.rsplit('/')
                    .next()
                    .map(|s| s.split('?').next().unwrap_or(s).to_string())
            });
        Ok((response, filename))
    }

    async fn paginate_messages(&self, path: &str, room_id: &str) -> Result<Vec<RocketMessage>> {
        let mut items = Vec::new();
        let mut offset = 0u64;
        loop {
            let mut params = HashMap::new();
            params.insert("roomId".to_string(), room_id.to_string());
            params.insert("count".to_string(), PAGE_SIZE.to_string());
            params.insert("offset".to_string(), offset.to_string());

            let response: RocketListResponse<RocketMessage> = self.get_json(path, &params).await?;
            let page_len = response.messages.len();
            items.extend(response.messages);
            offset += page_len as u64;
            if page_len < PAGE_SIZE as usize || offset >= response.total {
                break;
            }
        }
        Ok(items)
    }

    async fn paginate_typed<T: serde::de::DeserializeOwned + Default>(
        &self,
        path: &str,
        field: &str,
    ) -> Result<Vec<T>> {
        let mut items = Vec::new();
        let mut offset = 0u64;
        loop {
            let mut params = HashMap::new();
            params.insert("count".to_string(), PAGE_SIZE.to_string());
            params.insert("offset".to_string(), offset.to_string());

            let value = self.get_raw(path, &params).await?;
            let page: RocketListResponse<T> = serde_json::from_value(value)?;
            let page_len = page_len_for_field(&page, field);
            items.extend(extract_field(page, field));
            offset += page_len as u64;
            if page_len < PAGE_SIZE as usize {
                break;
            }
        }
        Ok(items)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &HashMap<String, String>,
    ) -> Result<T> {
        let value = self.get_raw(path, params).await?;
        serde_json::from_value(value).map_err(Into::into)
    }

    async fn get_raw(
        &self,
        path: &str,
        params: &HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        let url = self.session.url(path);
        debug!(%path, ?params, "RocketChat GET");
        let response = self
            .http
            .get(&url)
            .header("X-Auth-Token", &self.session.auth_token)
            .header("X-User-Id", &self.session.user_id)
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            warn!("RocketChat rate limited; backing off");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::rocketchat(
                Some(status),
                format!("RocketChat request failed: {text}"),
            ));
        }

        response.json().await.map_err(Into::into)
    }
}

fn page_len_for_field<T>(page: &RocketListResponse<T>, field: &str) -> usize {
    match field {
        "users" => page.users.len(),
        "channels" => page.channels.len(),
        "groups" => page.groups.len(),
        "ims" => page.ims.len(),
        "teams" => page.teams.len(),
        "emoji" => page.emoji.len(),
        "roles" => page.roles.len(),
        _ => 0,
    }
}

fn extract_field<T>(page: RocketListResponse<T>, field: &str) -> Vec<T> {
    match field {
        "users" => page.users,
        "channels" => page.channels,
        "groups" => page.groups,
        "ims" => page.ims,
        "teams" => page.teams,
        "emoji" => page.emoji,
        "roles" => page.roles,
        _ => Vec::new(),
    }
}

fn parse_filename(header: &str) -> Option<String> {
    let idx = header.find("filename=")?;
    let rest = &header[idx + "filename=".len()..];
    let rest = rest.trim_start_matches(['"', ' ']);
    let end = rest.find(['"', ';']).unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// Returns the best download URL and file metadata for a RocketChat attachment.
pub fn attachment_url_and_meta(
    attachment: &RocketAttachment,
    base_url: &str,
) -> Option<(String, String, String, Option<i64>)> {
    let candidates = [
        attachment.title_link.as_deref(),
        attachment.image_url.as_deref(),
        attachment.audio_url.as_deref(),
        attachment.video_url.as_deref(),
    ];
    let url = candidates.into_iter().flatten().next()?;
    let url = normalize_url(url, base_url);
    let filename = attachment
        .title
        .clone()
        .or_else(|| {
            url.rsplit('/')
                .next()
                .map(|s| s.split('?').next().unwrap_or(s).to_string())
        })
        .unwrap_or_else(|| "attachment".into());
    let content_type = attachment
        .content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".into());
    Some((url, filename, content_type, attachment.size))
}

/// Normalizes a relative RocketChat file URL.
pub fn normalize_url(url: &str, base_url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.into()
    } else {
        format!("{}{}", base_url.trim_end_matches('/'), url)
    }
}

/// Returns the display name for a RocketChat room.
pub fn room_display_name(room: &RocketRoom) -> String {
    room.fname
        .clone()
        .or_else(|| room.name.clone())
        .unwrap_or_else(|| room.id.clone())
}
