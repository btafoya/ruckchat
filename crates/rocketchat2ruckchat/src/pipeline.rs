//! Orchestrates the end-to-end migration from RocketChat to RuckChat.

use std::collections::HashMap;
use std::path::PathBuf;

use reqwest::Url;
use ruckchat_domain::CustomEmoji;
use ruckchat_id::{CustomEmojiId, OrganizationId, UserId};
use time::OffsetDateTime;
use tracing::{info, warn};

use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::mapping::MappingStore;
use crate::report::{Report, write_report};
use crate::rocket_chat::client::{RocketChatClient, attachment_url_and_meta, normalize_url};
use crate::rocket_chat::models::RocketMessage;
use crate::ruckchat::client::{MigrationData, RuckChatClient};
use crate::ruckchat::models::FileResponse;
use crate::transform::{
    RocketChatSource, build_migration_data, id_for, sanitize_channel_name, user_id_for_email,
};

/// Runs the migration and returns the path to the generated report.
pub async fn run(config: &ResolvedConfig) -> Result<PathBuf> {
    let mapping = MappingStore::open(&config.mapping_store)?;
    let rocket = RocketChatClient::new(&config.source.url, &config.source.auth).await?;
    let ruck = RuckChatClient::new(&config.target.url, &config.target.auth).await?;

    let target_org = target_org_meta(&config.target.url);
    let admin_user_id = user_id_for_email(&config.target.auth.login.email);

    let source = inventory_source(config, &rocket, &mapping, target_org).await?;

    let mut data = build_migration_data(config, &mapping, &source)?;

    if config.has_scope("emoji") {
        upload_emoji(
            config,
            &rocket,
            &ruck,
            &mapping,
            admin_user_id,
            &source,
            &mut data,
        )
        .await?;
    }

    if config.has_scope("files") {
        upload_message_files(
            config,
            &rocket,
            &ruck,
            &mapping,
            admin_user_id,
            &source,
            &mut data,
        )
        .await?;
    }

    let response = ruck
        .import_snapshot(
            OrganizationId::from_uuid(config.target.organization_id),
            &data,
            config.is_dry_run(),
        )
        .await?;

    mapping.put_checkpoint("import", None)?;

    let report = Report::from_run(config, &data, response);
    write_report(config, &report)
}

fn target_org_meta(url: &str) -> (String, String) {
    let host = Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| "migrated".into());
    let slug = sanitize_channel_name(&host);
    (host, slug)
}

async fn inventory_source(
    config: &ResolvedConfig,
    rocket: &RocketChatClient,
    mapping: &MappingStore,
    target_org: (String, String),
) -> Result<RocketChatSource> {
    let mut source = RocketChatSource {
        target_org_name: target_org.0,
        target_org_slug: target_org.1,
        users: Vec::new(),
        rooms: Vec::new(),
        messages: HashMap::new(),
        roles: Vec::new(),
        emoji: Vec::new(),
        teams: Vec::new(),
        team_members: HashMap::new(),
    };

    if config.has_scope("users") {
        info!("inventory: users");
        source.users = rocket.list_users().await?;
        mapping.put_checkpoint("users", None)?;
    }

    if config.has_scope("rooms") {
        info!("inventory: rooms");
        let mut rooms = Vec::new();
        rooms.extend(rocket.list_channels().await?);
        rooms.extend(rocket.list_groups().await?);
        rooms.extend(rocket.list_ims().await?);
        source.rooms = rooms;
        mapping.put_checkpoint("rooms", None)?;
    }

    if config.has_scope("roles") {
        info!("inventory: roles");
        source.roles = rocket.list_roles().await?;
        mapping.put_checkpoint("roles", None)?;
    }

    if config.has_scope("emoji") {
        info!("inventory: emoji");
        source.emoji = rocket.list_emoji().await?;
        mapping.put_checkpoint("emoji", None)?;
    }

    if config.has_scope("teams") {
        info!("inventory: teams");
        source.teams = rocket.list_teams().await?;
        mapping.put_checkpoint("teams", None)?;
    }

    if config.has_scope("messages") && !source.rooms.is_empty() {
        info!("inventory: messages");
        for room in &source.rooms {
            let messages = match room.room_type.as_str() {
                "c" => rocket.channel_history(&room.id).await,
                "p" | "v" => rocket.group_history(&room.id).await,
                "d" | "l" => rocket.im_history(&room.id).await,
                _ => continue,
            };
            match messages {
                Ok(list) => {
                    mapping.put_checkpoint("messages", Some(&room.id))?;
                    source.messages.insert(room.id.clone(), list);
                }
                Err(e) => {
                    warn!(room_id = %room.id, error = %e, "failed to fetch room messages");
                }
            }
        }
    }

    Ok(source)
}

async fn upload_emoji(
    config: &ResolvedConfig,
    rocket: &RocketChatClient,
    ruck: &RuckChatClient,
    mapping: &MappingStore,
    created_by: UserId,
    source: &RocketChatSource,
    data: &mut MigrationData,
) -> Result<()> {
    info!(count = source.emoji.len(), "uploading custom emoji");
    let org_id = OrganizationId::from_uuid(config.target.organization_id);

    for emoji in &source.emoji {
        if mapping.get_emoji(&emoji.id)?.is_some() {
            continue;
        }

        let extension = emoji.extension.clone().unwrap_or_else(|| "png".into());
        let url = format!("/emoji-custom/{}.{}", emoji.name, extension);
        let filename = format!("{}.{extension}", emoji.name);
        let mime = format!("image/{extension}");
        let full_url = normalize_url(&url, &config.source.url);

        let response = match rocket.download_file(&full_url).await {
            Ok((r, _)) => r,
            Err(e) => {
                warn!(shortcode = %emoji.name, error = %e, "emoji download failed");
                continue;
            }
        };
        let bytes = match response.bytes().await {
            Ok(b) => b.to_vec(),
            Err(e) => {
                warn!(shortcode = %emoji.name, error = %e, "emoji download failed");
                continue;
            }
        };
        if bytes.is_empty() {
            continue;
        }

        let file = match upload_bytes(ruck, org_id, created_by, &filename, &mime, bytes).await {
            Ok(f) => f,
            Err(e) => {
                warn!(shortcode = %emoji.name, error = %e, "emoji upload failed");
                continue;
            }
        };
        let file_id = file.id;
        data.files.push(file);

        let emoji_id = CustomEmojiId::from_uuid(id_for("emoji", &emoji.id));
        mapping.put_emoji(
            &emoji.id,
            &emoji_id.as_uuid().to_string(),
            &emoji.name,
            "create",
        )?;
        data.custom_emoji.push(CustomEmoji {
            id: emoji_id,
            organization_id: org_id,
            shortcode: emoji.name.clone(),
            file_id,
            created_by,
            created_at: OffsetDateTime::now_utc(),
        });
    }

    Ok(())
}

async fn upload_message_files(
    config: &ResolvedConfig,
    rocket: &RocketChatClient,
    ruck: &RuckChatClient,
    mapping: &MappingStore,
    default_uploader: UserId,
    source: &RocketChatSource,
    data: &mut MigrationData,
) -> Result<()> {
    info!("uploading message attachments");
    let org_id = OrganizationId::from_uuid(config.target.organization_id);

    for messages in source.messages.values() {
        for msg in messages {
            let uploader =
                lookup_user_by_rocket_id(&msg.user.id, source).unwrap_or(default_uploader);

            for attachment in &msg.attachments {
                let Some((url, filename, mime, _size)) =
                    attachment_url_and_meta(attachment, &config.source.url)
                else {
                    continue;
                };
                process_file_download(
                    rocket, ruck, mapping, org_id, uploader, msg, &url, &filename, &mime, data,
                )
                .await?;
            }

            if let Some(file) = &msg.file {
                let url = format!(
                    "/file-upload/{}/{}",
                    file.id,
                    file.name.clone().unwrap_or_default()
                );
                let filename = file.name.clone().unwrap_or_else(|| "attachment".into());
                let mime = file
                    .content_type
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".into());
                process_file_download(
                    rocket,
                    ruck,
                    mapping,
                    org_id,
                    uploader,
                    msg,
                    &normalize_url(&url, &config.source.url),
                    &filename,
                    &mime,
                    data,
                )
                .await?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_file_download(
    rocket: &RocketChatClient,
    ruck: &RuckChatClient,
    mapping: &MappingStore,
    org_id: OrganizationId,
    uploaded_by: UserId,
    msg: &RocketMessage,
    url: &str,
    filename: &str,
    mime: &str,
    data: &mut MigrationData,
) -> Result<()> {
    let rocket_file_id = format!("{}-{}", msg.id, url);
    if mapping.get_file(&rocket_file_id)?.is_some() {
        return Ok(());
    }

    let response = match rocket.download_file(url).await {
        Ok((r, _)) => r,
        Err(e) => {
            warn!(url, error = %e, "file download failed");
            return Ok(());
        }
    };
    let bytes = match response.bytes().await {
        Ok(b) => b.to_vec(),
        Err(e) => {
            warn!(url, error = %e, "file download failed");
            return Ok(());
        }
    };
    if bytes.is_empty() {
        return Ok(());
    }

    let file = upload_bytes(ruck, org_id, uploaded_by, filename, mime, bytes).await?;
    mapping.put_file(
        &rocket_file_id,
        &file.id.as_uuid().to_string(),
        Some(&file.storage_path),
        "create",
    )?;

    let message_id = id_for("message", &msg.id);
    data.message_files
        .push(crate::ruckchat::client::MessageFileLink {
            message_id: ruckchat_id::MessageId::from_uuid(message_id),
            file_id: file.id,
        });
    data.files.push(file);

    Ok(())
}

async fn upload_bytes(
    ruck: &RuckChatClient,
    org_id: OrganizationId,
    _uploaded_by: UserId,
    filename: &str,
    mime: &str,
    bytes: Vec<u8>,
) -> Result<ruckchat_domain::File> {
    let resp: FileResponse = ruck.upload_file(org_id, filename, mime, bytes).await?;
    ruck.get_file_metadata(resp.id).await
}

fn lookup_user_by_rocket_id(rocket_id: &str, source: &RocketChatSource) -> Option<UserId> {
    source
        .users
        .iter()
        .find(|u| u.id == rocket_id)
        .and_then(|u| u.emails.first().map(|e| user_id_for_email(&e.address)))
}
