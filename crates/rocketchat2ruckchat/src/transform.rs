//! RocketChat MongoDB dump → RuckChat `MigrationData` transformation.

use std::collections::{HashMap, HashSet};

use rand::RngCore;
use ruckchat_domain::{
    Channel, ChannelMembership, DirectMessageConversation, Organization, OrganizationMembership,
    OrganizationSettings, Reaction, Role, User,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, MessageId, OrganizationId, UserId};
use ruckchat_migrate::{MIGRATION_VERSION, MigrationData, MigrationMessage};
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::mapping::MappingStore;
use crate::mongo_source::{
    RocketMessage, RocketRoom, RocketSubscription, RocketUser, bson_datetime_or,
};

/// Namespace used for deterministic UUID generation.
const RUCKCHAT_NAMESPACE: Uuid = Uuid::from_u128(0x72_75_63_6b_63_68_61_74_20_6d_69_67_72_61_74_65);

/// Generates a deterministic UUID from a RocketChat identifier and a category.
pub fn id_for(category: &str, rocket_id: &str) -> Uuid {
    Uuid::new_v5(
        &RUCKCHAT_NAMESPACE,
        format!("{category}:{rocket_id}").as_bytes(),
    )
}

/// Generates a deterministic user id from an email address.
pub fn user_id_for_email(email: &str) -> UserId {
    UserId::from_uuid(id_for("user_email", email))
}

/// Sanitizes a RocketChat room name into a valid RuckChat channel name.
pub fn sanitize_channel_name(name: &str) -> String {
    let mut out: String = name
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    out = out.trim_matches('-').to_string();
    if out.is_empty() {
        out = "migrated".into();
    }
    out
}

/// Determines whether a RocketChat room type is a direct message. Livechat
/// rooms (`t: "l"`) are filtered out at the Mongo query level and never
/// reach this function.
fn is_direct_message(room_type: &str) -> bool {
    room_type == "d"
}

fn room_display_name(room: &RocketRoom) -> String {
    room.fname
        .clone()
        .or_else(|| room.name.clone())
        .unwrap_or_else(|| room.id.clone())
}

fn generate_temp_password() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_password(password: &str) -> String {
    use argon2::password_hash::{SaltString, rand_core::OsRng};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 hashing does not fail for a well-formed random password")
        .to_string()
}

/// Context shared across transformation helpers.
struct TransformContext {
    target_org_id: OrganizationId,
    target_org_name: String,
    target_org_slug: String,
    admin_user_id: UserId,
    user_map: HashMap<String, UserId>,
    username_map: HashMap<String, UserId>,
    room_map: HashMap<String, Uuid>,
    now: OffsetDateTime,
}

impl TransformContext {
    fn user_id(&self, rocket_id: &str) -> Option<UserId> {
        self.user_map.get(rocket_id).copied()
    }

    fn user_by_username(&self, username: &str) -> Option<UserId> {
        self.username_map.get(username).copied()
    }

    fn room_id(&self, rocket_id: &str) -> Option<Uuid> {
        self.room_map.get(rocket_id).copied()
    }
}

/// Source data fetched directly from the RocketChat MongoDB dump.
#[derive(Debug, Default)]
pub struct MongoSourceData {
    /// Users.
    pub users: Vec<RocketUser>,
    /// Rooms (channels, private groups, direct messages, discussions).
    pub rooms: Vec<RocketRoom>,
    /// Subscriptions: one row per user per room, the authoritative
    /// membership source for every room type.
    pub subscriptions: Vec<RocketSubscription>,
    /// Messages, keyed by room identifier.
    pub messages: HashMap<String, Vec<RocketMessage>>,
}

/// Plaintext temporary passwords generated for newly migrated users, keyed
/// by their resolved [`UserId`]. Held only in memory for the lifetime of the
/// run so `--send-emails` can deliver them; never written to the mapping
/// store or the migration report.
pub type TempPasswords = HashMap<UserId, String>;

/// Transforms Mongo source data into a RuckChat `MigrationData` snapshot.
///
/// This function performs only in-memory transformation; file and emoji
/// uploads are handled separately by the pipeline, which has access to the
/// Mongo GridFS reader and the Postgres target's upload directory.
///
/// # Errors
///
/// Returns an error when the mapping store cannot be read or written.
pub fn build_migration_data(
    config: &ResolvedConfig,
    mapping: &MappingStore,
    source: &MongoSourceData,
    existing_users_by_email: &HashMap<String, Uuid>,
) -> Result<(MigrationData, TempPasswords)> {
    let target_org_id = OrganizationId::from_uuid(config.target.organization_id);
    let admin_email_key = config.target.admin_email.to_ascii_lowercase();
    let admin_user_id = existing_users_by_email
        .get(&admin_email_key)
        .copied()
        .map(UserId::from_uuid)
        .unwrap_or_else(|| user_id_for_email(&config.target.admin_email));

    let mut ctx = TransformContext {
        target_org_id,
        target_org_name: config.target.organization_name.clone(),
        target_org_slug: config.target.organization_slug.clone(),
        admin_user_id,
        user_map: HashMap::new(),
        username_map: HashMap::new(),
        room_map: HashMap::new(),
        now: OffsetDateTime::now_utc(),
    };

    let (users, temp_passwords) = transform_users(
        &mut ctx,
        mapping,
        &source.users,
        &config.target.admin_email,
        existing_users_by_email,
    )?;
    let organization = build_organization(&ctx);
    let organization_memberships = build_memberships(&ctx, &source.users);
    let organization_settings = vec![OrganizationSettings::new(target_org_id)];

    let subs_by_room = group_subscriptions_by_room(&ctx, &source.subscriptions);

    let (channels, dms) = transform_rooms(&mut ctx, mapping, &source.rooms, &subs_by_room)?;
    let channel_memberships = build_channel_memberships(&ctx, &source.rooms, &subs_by_room);

    let (messages, reactions) = transform_messages(&ctx, mapping, &source.rooms, &source.messages)?;

    Ok((
        MigrationData {
            version: MIGRATION_VERSION,
            exported_at: OffsetDateTime::now_utc(),
            users,
            organizations: vec![organization],
            organization_memberships,
            organization_settings,
            organization_roles: Vec::new(),
            permissions: Vec::new(),
            role_permissions: Vec::new(),
            custom_emoji: Vec::new(),
            teams: Vec::new(),
            team_memberships: Vec::new(),
            team_rooms: Vec::new(),
            channels,
            channel_memberships,
            direct_message_conversations: dms,
            messages,
            reactions,
            files: Vec::new(),
            message_files: Vec::new(),
        },
        temp_passwords,
    ))
}

fn transform_users(
    ctx: &mut TransformContext,
    mapping: &MappingStore,
    users: &[RocketUser],
    admin_email: &str,
    existing_users_by_email: &HashMap<String, Uuid>,
) -> Result<(Vec<User>, TempPasswords)> {
    let mut result = Vec::with_capacity(users.len() + 1);
    let mut temp_passwords = TempPasswords::new();
    let admin_id = ctx.admin_user_id;
    let admin_key = admin_email.to_ascii_lowercase();

    for user in users {
        let email = user
            .emails
            .first()
            .map(|e| e.address.clone())
            .unwrap_or_else(|| format!("{}@rocketchat.local", user.username));
        let email_key = email.to_ascii_lowercase();
        let is_admin = email_key == admin_key;
        let existing = existing_users_by_email.get(&email_key).copied();

        let id = if is_admin {
            admin_id
        } else if let Some(existing_id) = existing {
            UserId::from_uuid(existing_id)
        } else {
            UserId::from_uuid(id_for("user", &user.id))
        };

        ctx.user_map.insert(user.id.clone(), id);
        ctx.username_map.insert(user.username.clone(), id);

        // A pre-existing account's credentials are never touched; the
        // password_hash value below is discarded by ON CONFLICT DO NOTHING.
        let already_exists = existing.is_some();
        let password_hash = if already_exists {
            "!unused".to_string()
        } else {
            let temp_password = generate_temp_password();
            let hash = hash_password(&temp_password);
            temp_passwords.insert(id, temp_password);
            hash
        };

        let display_name = user.name.clone().unwrap_or_else(|| user.username.clone());
        let deactivated_at = if !user.active { Some(ctx.now) } else { None };

        let action = if mapping.get_user(&user.id)?.is_some() {
            "update"
        } else {
            "create"
        };
        mapping.put_user(&user.id, &id.as_uuid().to_string(), Some(&email), action)?;

        result.push(User {
            id,
            email,
            display_name,
            password_hash,
            avatar_url: None,
            is_server_admin: is_admin,
            theme: "system".to_string(),
            deactivated_at,
            created_at: bson_datetime_or(user.created_at, ctx.now),
            updated_at: ctx.now,
        });
    }

    // Ensure the target organization's admin/owner is present even when no
    // RocketChat user matched them by email.
    if !result.iter().any(|u| u.id == admin_id) {
        let already_exists = existing_users_by_email.contains_key(&admin_key);
        let password_hash = if already_exists {
            "!unused".to_string()
        } else {
            let temp_password = generate_temp_password();
            let hash = hash_password(&temp_password);
            temp_passwords.insert(admin_id, temp_password);
            hash
        };

        result.push(User {
            id: admin_id,
            email: admin_email.to_string(),
            display_name: "Migration Admin".into(),
            password_hash,
            avatar_url: None,
            is_server_admin: true,
            theme: "system".to_string(),
            deactivated_at: None,
            created_at: ctx.now,
            updated_at: ctx.now,
        });
    }

    Ok((result, temp_passwords))
}

fn build_organization(ctx: &TransformContext) -> Organization {
    Organization {
        id: ctx.target_org_id,
        name: ctx.target_org_name.clone(),
        slug: ctx.target_org_slug.clone(),
        owner_id: ctx.admin_user_id,
        created_at: ctx.now,
        updated_at: ctx.now,
    }
}

fn build_memberships(ctx: &TransformContext, users: &[RocketUser]) -> Vec<OrganizationMembership> {
    let mut result = Vec::with_capacity(users.len() + 1);
    let mut seen: HashSet<UserId> = HashSet::new();

    for user in users {
        let Some(user_id) = ctx.user_id(&user.id) else {
            continue;
        };
        if !seen.insert(user_id) {
            continue;
        }
        let role = if user_id == ctx.admin_user_id {
            Role::Owner
        } else {
            Role::Member
        };
        result.push(OrganizationMembership {
            user_id,
            organization_id: ctx.target_org_id,
            role,
            joined_at: ctx.now,
        });
    }

    if seen.insert(ctx.admin_user_id) {
        result.push(OrganizationMembership {
            user_id: ctx.admin_user_id,
            organization_id: ctx.target_org_id,
            role: Role::Owner,
            joined_at: ctx.now,
        });
    }

    result
}

/// Groups subscriptions by room, resolving each to an already-migrated
/// [`UserId`]. Must run after [`transform_users`] has populated the user maps.
fn group_subscriptions_by_room(
    ctx: &TransformContext,
    subscriptions: &[RocketSubscription],
) -> HashMap<String, Vec<UserId>> {
    let mut map: HashMap<String, Vec<UserId>> = HashMap::new();
    for sub in subscriptions {
        let Some(user_id) = ctx.user_id(&sub.u.id).or_else(|| {
            sub.u
                .username
                .as_deref()
                .and_then(|username| ctx.user_by_username(username))
        }) else {
            continue;
        };
        map.entry(sub.rid.clone()).or_default().push(user_id);
    }
    map
}

fn transform_rooms(
    ctx: &mut TransformContext,
    mapping: &MappingStore,
    rooms: &[RocketRoom],
    subs_by_room: &HashMap<String, Vec<UserId>>,
) -> Result<(Vec<Channel>, Vec<DirectMessageConversation>)> {
    // Pass 1: assign every room a deterministic id up front so a discussion's
    // `prid` parent link resolves regardless of iteration order.
    for room in rooms {
        let uuid = if is_direct_message(&room.room_type) {
            id_for("dm", &room.id)
        } else {
            id_for("channel", &room.id)
        };
        ctx.room_map.insert(room.id.clone(), uuid);
    }

    let mut channels = Vec::new();
    let mut dms = Vec::new();

    for room in rooms {
        let room_uuid = ctx
            .room_id(&room.id)
            .expect("assigned to every room in pass 1 above");

        if is_direct_message(&room.room_type) {
            let mut member_ids: Vec<UserId> =
                subs_by_room.get(&room.id).cloned().unwrap_or_default();
            member_ids.sort_unstable();
            member_ids.dedup();
            if member_ids.len() < 2 {
                member_ids.push(ctx.admin_user_id);
                member_ids.sort_unstable();
                member_ids.dedup();
            }
            if member_ids.len() < 2 {
                continue;
            }

            let action = if mapping.get_room(&room.id)?.is_some() {
                "update"
            } else {
                "create"
            };
            mapping.put_room(
                &room.id,
                &room_uuid.to_string(),
                &room.room_type,
                "dm",
                action,
            )?;

            dms.push(DirectMessageConversation {
                id: DirectMessageConversationId::from_uuid(room_uuid),
                organization_id: ctx.target_org_id,
                member_ids,
                created_at: bson_datetime_or(room.ts, ctx.now),
            });
            continue;
        }

        let name = sanitize_channel_name(&room_display_name(room));
        let is_private = room.room_type == "p";
        let created_by = room
            .u
            .as_ref()
            .and_then(|u| ctx.user_id(&u.id))
            .unwrap_or(ctx.admin_user_id);
        let archived_at = if room.archived { Some(ctx.now) } else { None };
        let parent_channel_id = room
            .prid
            .as_deref()
            .and_then(|parent| ctx.room_id(parent))
            .map(ChannelId::from_uuid);

        let action = if mapping.get_room(&room.id)?.is_some() {
            "update"
        } else {
            "create"
        };
        mapping.put_room(
            &room.id,
            &room_uuid.to_string(),
            &room.room_type,
            "channel",
            action,
        )?;

        channels.push(Channel {
            id: ChannelId::from_uuid(room_uuid),
            organization_id: ctx.target_org_id,
            name,
            topic: room.topic.clone(),
            purpose: room.description.clone(),
            is_private,
            created_by,
            created_at: bson_datetime_or(room.ts, ctx.now),
            archived_at,
            parent_channel_id,
        });
    }

    Ok((channels, dms))
}

fn build_channel_memberships(
    ctx: &TransformContext,
    rooms: &[RocketRoom],
    subs_by_room: &HashMap<String, Vec<UserId>>,
) -> Vec<ChannelMembership> {
    let mut result = Vec::new();
    let mut seen: HashSet<(UserId, Uuid)> = HashSet::new();

    for room in rooms {
        if is_direct_message(&room.room_type) {
            continue;
        }
        let Some(channel_id) = ctx.room_id(&room.id) else {
            continue;
        };

        let members = subs_by_room.get(&room.id).cloned().unwrap_or_default();
        let creator = room.u.as_ref().and_then(|u| ctx.user_id(&u.id));

        for user_id in members.into_iter().chain(creator) {
            if seen.insert((user_id, channel_id)) {
                result.push(ChannelMembership {
                    user_id,
                    channel_id: ChannelId::from_uuid(channel_id),
                    joined_at: ctx.now,
                });
            }
        }
    }
    result
}

fn transform_messages(
    ctx: &TransformContext,
    mapping: &MappingStore,
    rooms: &[RocketRoom],
    room_messages: &HashMap<String, Vec<RocketMessage>>,
) -> Result<(Vec<MigrationMessage>, Vec<Reaction>)> {
    let mut messages = Vec::new();
    let mut reactions = Vec::new();
    let mut message_id_map: HashMap<String, MessageId> = HashMap::new();

    for room in rooms {
        let Some(conversation_uuid) = ctx.room_id(&room.id) else {
            continue;
        };
        let conversation_type = if is_direct_message(&room.room_type) {
            "dm".to_string()
        } else {
            "channel".to_string()
        };

        for msg in room_messages.get(&room.id).unwrap_or(&Vec::new()) {
            // System messages (t: "au", "uj", "r", "message_pinned", ...)
            // have no equivalent in RuckChat and are skipped entirely.
            if msg.system_type.is_some() {
                continue;
            }

            let Some(author_id) = ctx.user_id(&msg.u.id).or_else(|| {
                msg.u
                    .username
                    .as_deref()
                    .and_then(|username| ctx.user_by_username(username))
            }) else {
                warn!(rocket_user_id = %msg.u.id, "message author not found; skipping");
                continue;
            };

            let message_id = MessageId::from_uuid(id_for("message", &msg.id));
            message_id_map.insert(msg.id.clone(), message_id);

            let parent_id = msg
                .tmid
                .as_ref()
                .and_then(|pid| message_id_map.get(pid).copied());

            let created_at = bson_datetime_or(msg.ts, ctx.now);
            let updated_at = bson_datetime_or(msg.edited_at, created_at);

            let action = if mapping.get_message(&msg.id)?.is_some() {
                "update"
            } else {
                "create"
            };
            mapping.put_message(&msg.id, &message_id.as_uuid().to_string(), action)?;

            messages.push(MigrationMessage {
                id: message_id,
                conversation_id: conversation_uuid,
                conversation_type: conversation_type.clone(),
                parent_id,
                author_id,
                content: msg.msg.clone(),
                mentioned_user_ids: vec![],
                created_at,
                updated_at,
                deleted_at: None,
            });

            for (emoji, entry) in &msg.reactions {
                for username in &entry.usernames {
                    let Some(user_id) = ctx.user_by_username(username) else {
                        continue;
                    };
                    reactions.push(Reaction {
                        message_id,
                        user_id,
                        emoji: emoji.clone(),
                        created_at,
                    });
                }
            }
        }
    }

    Ok((messages, reactions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_id_is_stable() {
        let a = id_for("user", "abc123");
        let b = id_for("user", "abc123");
        assert_eq!(a, b);
        assert_ne!(a, id_for("user", "xyz"));
    }

    #[test]
    fn sanitize_channel_name_replaces_invalid_chars() {
        assert_eq!(sanitize_channel_name("General Chat"), "general-chat");
        assert_eq!(sanitize_channel_name("-room-"), "room");
    }

    #[test]
    fn temp_password_and_hash_round_trip() {
        let password = generate_temp_password();
        assert_eq!(password.len(), 32);
        let hash = hash_password(&password);
        assert!(hash.starts_with("$argon2"));
    }
}
