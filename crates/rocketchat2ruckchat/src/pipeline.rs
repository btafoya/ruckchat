//! Orchestrates the end-to-end migration from a RocketChat MongoDB dump to a
//! RuckChat PostgreSQL database.

use std::collections::HashMap;
use std::path::PathBuf;

use ruckchat_domain::{CustomEmoji, File as RuckFile};
use ruckchat_email::EmailClient;
use ruckchat_id::{FileId, MessageId, OrganizationId, UserId};
use ruckchat_migrate::{MessageFileLink, MigrationData};
use time::OffsetDateTime;
use tracing::{info, warn};

use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::mapping::MappingStore;
use crate::mongo_source::{MongoSource, RocketMessageFileRef};
use crate::report::{CredentialEmailOutcome, Report, write_report};
use crate::target::PostgresTarget;
use crate::transform::{MongoSourceData, TempPasswords, build_migration_data, id_for};

/// Runs the migration and returns the path to the generated report.
pub async fn run(config: &ResolvedConfig) -> Result<PathBuf> {
    let mapping = MappingStore::open(&config.mapping_store)?;
    let source_db = MongoSource::connect(&config.source.mongo_uri, &config.source.database).await?;
    let target_db =
        PostgresTarget::connect(&config.target.database_url, &config.target.upload_directory)
            .await?;

    if let Some(version) = source_db.migration_version().await? {
        info!(version, "source RocketChat schema migration version");
    }

    let existing_users_by_email = target_db.existing_users_by_email().await?;

    let source = inventory_source(config, &source_db, &mapping).await?;
    let (mut data, temp_passwords) =
        build_migration_data(config, &mapping, &source, &existing_users_by_email)?;

    if config.has_scope("emoji") {
        upload_emoji(config, &source_db, &target_db, &mapping, &mut data).await?;
    }

    if config.has_scope("files") {
        upload_message_files(&source_db, &target_db, &mapping, &source, &mut data).await?;
    }

    let counts = target_db.import(&data, config.is_dry_run()).await?;
    mapping.put_checkpoint("import", None)?;

    let credential_emails = if config.send_emails && !config.is_dry_run() {
        send_credential_emails(config, &temp_passwords, &data).await
    } else {
        Vec::new()
    };

    let report = Report::from_run(config, &data, counts, credential_emails);
    write_report(config, &report)
}

async fn inventory_source(
    config: &ResolvedConfig,
    db: &MongoSource,
    mapping: &MappingStore,
) -> Result<MongoSourceData> {
    let mut source = MongoSourceData::default();

    if config.has_scope("users") {
        info!("inventory: users");
        source.users = db.list_users().await?;
        mapping.put_checkpoint("users", None)?;
    }

    if config.has_scope("channels") {
        info!("inventory: rooms and subscriptions");
        source.rooms = db.list_rooms().await?;
        source.subscriptions = db.list_subscriptions().await?;
        mapping.put_checkpoint("channels", None)?;
    }

    if config.has_scope("messages") && !source.rooms.is_empty() {
        info!("inventory: messages");
        for room in &source.rooms {
            match db.list_messages_for_room(&room.id).await {
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
    source_db: &MongoSource,
    target_db: &PostgresTarget,
    mapping: &MappingStore,
    data: &mut MigrationData,
) -> Result<()> {
    let org_id = OrganizationId::from_uuid(config.target.organization_id);
    let created_by = data.organizations[0].owner_id;
    let emoji_list = source_db.list_custom_emoji().await?;
    info!(count = emoji_list.len(), "migrating custom emoji");

    for emoji in &emoji_list {
        if mapping.get_emoji(&emoji.id)?.is_some() {
            continue;
        }

        // RocketChat commonly reuses the custom-emoji document's own id as
        // its upload id; this is unverified against real data (no instance
        // surveyed had any custom emoji) so failures are reported, not
        // silently swallowed. See docs/DESIGN-RocketChat-Migration.md.
        let Some(upload) = source_db.find_upload(&emoji.id).await? else {
            warn!(shortcode = %emoji.name, "no matching upload found for custom emoji; skipping");
            continue;
        };
        let Some(bytes) = source_db
            .download_gridfs_bytes(&upload.store, &upload.id)
            .await?
        else {
            warn!(shortcode = %emoji.name, store = %upload.store, "custom emoji is not GridFS-backed; skipping");
            continue;
        };

        let file_id = FileId::from_uuid(id_for("file", &upload.id));
        let storage_path = target_db.write_file_bytes(file_id, &bytes).await?;
        let extension = emoji.extension.clone().unwrap_or_else(|| "png".into());

        mapping.put_file(
            &upload.id,
            &file_id.as_uuid().to_string(),
            Some(&storage_path),
            "create",
        )?;
        data.files.push(RuckFile {
            id: file_id,
            organization_id: org_id,
            uploaded_by: created_by,
            file_name: format!("{}.{extension}", emoji.name),
            mime_type: upload
                .content_type
                .clone()
                .unwrap_or_else(|| format!("image/{extension}")),
            size_bytes: i64::try_from(bytes.len()).unwrap_or(i64::MAX),
            storage_path,
            thumbnail_path: None,
            created_at: OffsetDateTime::now_utc(),
        });

        let emoji_id = ruckchat_id::CustomEmojiId::from_uuid(id_for("emoji", &emoji.id));
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
    source_db: &MongoSource,
    target_db: &PostgresTarget,
    mapping: &MappingStore,
    source: &MongoSourceData,
    data: &mut MigrationData,
) -> Result<()> {
    let org_id = data.organizations[0].id;
    let default_uploader = data.organizations[0].owner_id;
    info!("migrating message attachments");

    let author_by_message: HashMap<MessageId, UserId> =
        data.messages.iter().map(|m| (m.id, m.author_id)).collect();

    for messages in source.messages.values() {
        for msg in messages {
            if msg.system_type.is_some() {
                continue;
            }
            let Some(message_id) = mapping
                .get_message(&msg.id)?
                .and_then(|s| s.parse().ok())
                .map(MessageId::from_uuid)
            else {
                continue;
            };
            let uploader = author_by_message
                .get(&message_id)
                .copied()
                .unwrap_or(default_uploader);

            let refs: Vec<&RocketMessageFileRef> =
                msg.file.iter().chain(msg.files.iter()).collect();
            for file_ref in refs {
                process_attachment(
                    source_db,
                    target_db,
                    mapping,
                    org_id,
                    uploader,
                    message_id,
                    &file_ref.id,
                    data,
                )
                .await?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_attachment(
    source_db: &MongoSource,
    target_db: &PostgresTarget,
    mapping: &MappingStore,
    org_id: OrganizationId,
    uploaded_by: UserId,
    message_id: MessageId,
    upload_id: &str,
    data: &mut MigrationData,
) -> Result<()> {
    if let Some(existing) = mapping.get_file(upload_id)? {
        if let Ok(file_uuid) = existing.parse() {
            data.message_files.push(MessageFileLink {
                message_id,
                file_id: FileId::from_uuid(file_uuid),
            });
        }
        return Ok(());
    }

    let Some(upload) = source_db.find_upload(upload_id).await? else {
        warn!(upload_id, "referenced upload not found; skipping");
        return Ok(());
    };
    let Some(bytes) = source_db
        .download_gridfs_bytes(&upload.store, &upload.id)
        .await?
    else {
        warn!(upload_id, store = %upload.store, "file is not GridFS-backed; skipping");
        return Ok(());
    };

    let file_id = FileId::from_uuid(id_for("file", &upload.id));
    let storage_path = target_db.write_file_bytes(file_id, &bytes).await?;

    mapping.put_file(
        upload_id,
        &file_id.as_uuid().to_string(),
        Some(&storage_path),
        "create",
    )?;

    data.files.push(RuckFile {
        id: file_id,
        organization_id: org_id,
        uploaded_by,
        file_name: upload.name.clone().unwrap_or_else(|| "attachment".into()),
        mime_type: upload
            .content_type
            .clone()
            .unwrap_or_else(|| "application/octet-stream".into()),
        size_bytes: upload
            .size
            .unwrap_or_else(|| i64::try_from(bytes.len()).unwrap_or(i64::MAX)),
        storage_path,
        thumbnail_path: None,
        created_at: OffsetDateTime::now_utc(),
    });
    data.message_files.push(MessageFileLink {
        message_id,
        file_id,
    });

    Ok(())
}

async fn send_credential_emails(
    config: &ResolvedConfig,
    temp_passwords: &TempPasswords,
    data: &MigrationData,
) -> Vec<CredentialEmailOutcome> {
    let Some(email_config) = &config.email else {
        warn!("--send-emails was set but no email configuration was provided; skipping");
        return Vec::new();
    };
    let client = EmailClient::new(&ruckchat_email::EmailConfig {
        server_token: email_config.server_token.clone(),
        from_address: email_config.from_address.clone(),
    });

    let email_by_id: HashMap<UserId, &str> = data
        .users
        .iter()
        .map(|u| (u.id, u.email.as_str()))
        .collect();

    let mut outcomes = Vec::with_capacity(temp_passwords.len());
    for (user_id, temp_password) in temp_passwords {
        let Some(&email) = email_by_id.get(user_id) else {
            continue;
        };
        match client
            .send_migration_credentials(email, temp_password)
            .await
        {
            Ok(()) => outcomes.push(CredentialEmailOutcome {
                email: email.to_string(),
                error: None,
            }),
            Err(err) => {
                warn!(email, error = %err, "failed to send migration credential email");
                outcomes.push(CredentialEmailOutcome {
                    email: email.to_string(),
                    error: Some(err.to_string()),
                });
            }
        }
    }
    outcomes
}
