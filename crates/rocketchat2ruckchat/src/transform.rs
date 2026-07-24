//! RocketChat → RuckChat snapshot transformation.

use std::collections::{HashMap, HashSet};

use ruckchat_domain::{
    Channel, ChannelMembership, DirectMessageConversation, Organization, OrganizationMembership,
    OrganizationRole, OrganizationSettings, Permission, Reaction, Role, Team, TeamMembership,
    TeamRole, TeamRoom, User,
};
use ruckchat_id::{ChannelId, DirectMessageConversationId, MessageId, OrganizationId, UserId};
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

use crate::config::ResolvedConfig;
use crate::error::Result;
use crate::mapping::MappingStore;
use crate::rocket_chat::client::room_display_name;
use crate::rocket_chat::models::{RocketMessage, RocketRole, RocketRoom, RocketTeam, RocketUser};
use crate::ruckchat::client::{MigrationData, MigrationMessage, RuckRolePermission};

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

/// Parses an ISO 8601 / RFC 3339 timestamp from RocketChat.
fn parse_timestamp(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
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

/// Determines whether a RocketChat room type is a direct message.
fn is_direct_message(room_type: &str) -> bool {
    room_type == "d" || room_type == "l"
}

/// Context shared across transformation helpers.
struct TransformContext {
    target_org_id: OrganizationId,
    target_org_name: String,
    target_org_slug: String,
    admin_user_id: UserId,
    options: crate::config::OptionsConfig,
    user_map: HashMap<String, UserId>,
    username_map: HashMap<String, UserId>,
    room_map: HashMap<String, Uuid>,
    team_map: HashMap<String, Uuid>,
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

    fn team_id(&self, rocket_id: &str) -> Option<Uuid> {
        self.team_map.get(rocket_id).copied()
    }
}

/// Source data fetched from RocketChat.
#[derive(Debug, Default, Clone)]
pub struct RocketChatSource {
    /// Target organization display name.
    pub target_org_name: String,
    /// Target organization slug.
    pub target_org_slug: String,
    /// Users fetched from RocketChat.
    pub users: Vec<RocketUser>,
    /// Rooms fetched from RocketChat.
    pub rooms: Vec<RocketRoom>,
    /// Messages fetched from RocketChat, keyed by room identifier.
    pub messages: HashMap<String, Vec<RocketMessage>>,
    /// Roles fetched from RocketChat.
    pub roles: Vec<RocketRole>,
    /// Custom emoji fetched from RocketChat.
    pub emoji: Vec<crate::rocket_chat::models::RocketEmoji>,
    /// Teams fetched from RocketChat.
    pub teams: Vec<RocketTeam>,
    /// Team members keyed by team identifier.
    pub team_members: HashMap<String, Vec<String>>,
}

/// Transforms source data into a RuckChat `MigrationData` snapshot.
///
/// This function performs only in-memory transformation; file and emoji
/// uploads are handled separately by the pipeline.
pub fn build_migration_data(
    config: &ResolvedConfig,
    mapping: &MappingStore,
    source: &RocketChatSource,
) -> Result<MigrationData> {
    let target_org_id = ruckchat_id::OrganizationId::from_uuid(config.target.organization_id);
    let admin_user_id = user_id_for_email(&config.target.auth.login.email);

    let mut ctx = TransformContext {
        target_org_id,
        target_org_name: source.target_org_name.clone(),
        target_org_slug: source.target_org_slug.clone(),
        admin_user_id,
        options: config.options.clone(),
        user_map: HashMap::new(),
        username_map: HashMap::new(),
        room_map: HashMap::new(),
        team_map: HashMap::new(),
        now: OffsetDateTime::now_utc(),
    };

    let users = transform_users(
        &mut ctx,
        mapping,
        &source.users,
        &config.target.auth.login.email,
    )?;
    let organization = build_organization(&ctx);
    let organization_memberships = build_memberships(&ctx, mapping, &source.users);
    let organization_settings = vec![OrganizationSettings::new(target_org_id)];

    let (organization_roles, permissions, role_permissions) =
        transform_roles_and_permissions(&mut ctx, mapping, &source.roles)?;

    let teams = transform_teams(&mut ctx, mapping, &source.teams)?;
    let team_memberships = transform_team_memberships(&ctx, &source.team_members);

    let (channels, dms) = transform_rooms(&mut ctx, mapping, &source.rooms)?;
    let channel_memberships = build_channel_memberships(&ctx, &source.rooms);
    let team_rooms = build_team_rooms(&ctx, &source.rooms);

    let (messages, reactions) = transform_messages(&ctx, mapping, &source.rooms, &source.messages)?;

    Ok(MigrationData {
        version: crate::ruckchat::client::MIGRATION_VERSION,
        exported_at: OffsetDateTime::now_utc(),
        users,
        organizations: vec![organization],
        organization_memberships,
        organization_settings,
        organization_roles,
        permissions,
        role_permissions,
        custom_emoji: Vec::new(),
        teams,
        team_memberships,
        team_rooms,
        channels,
        channel_memberships,
        direct_message_conversations: dms,
        messages,
        reactions,
        files: Vec::new(),
        message_files: Vec::new(),
    })
}

fn transform_users(
    ctx: &mut TransformContext,
    mapping: &MappingStore,
    users: &[RocketUser],
    admin_email: &str,
) -> Result<Vec<User>> {
    let mut result = Vec::with_capacity(users.len() + 1);
    let admin_id = user_id_for_email(admin_email);

    for user in users {
        // If a RocketChat user shares the admin email, reuse the admin id so
        // the snapshot does not contain two users with the same email address.
        let id = user
            .emails
            .iter()
            .find(|e| e.address.eq_ignore_ascii_case(admin_email))
            .map(|_| admin_id)
            .unwrap_or_else(|| UserId::from_uuid(id_for("user", &user.id)));

        ctx.user_map.insert(user.id.clone(), id);
        ctx.username_map.insert(user.username.clone(), id);

        let email = user
            .emails
            .first()
            .map(|e| e.address.clone())
            .unwrap_or_else(|| format!("{}@rocketchat.local", user.username));
        let display_name = user.name.clone().unwrap_or_else(|| user.username.clone());
        let deactivated_at = if user.deleted_at.is_some() || !user.active {
            Some(ctx.now)
        } else {
            None
        };

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
            password_hash: "!notset".into(),
            avatar_url: user.avatar_url.clone(),
            is_server_admin: false,
            deactivated_at,
            created_at: ctx.now,
            updated_at: ctx.now,
        });
    }

    // Ensure the target organization owner is present in the user list.
    if !result.iter().any(|u| u.id == admin_id) {
        result.push(User {
            id: admin_id,
            email: admin_email.into(),
            display_name: "Migration Admin".into(),
            password_hash: "!notset".into(),
            avatar_url: None,
            is_server_admin: true,
            deactivated_at: None,
            created_at: ctx.now,
            updated_at: ctx.now,
        });
    }

    Ok(result)
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

fn build_memberships(
    ctx: &TransformContext,
    mapping: &MappingStore,
    users: &[RocketUser],
) -> Vec<OrganizationMembership> {
    let mut result = Vec::with_capacity(users.len());
    for user in users {
        let Some(user_id) = ctx.user_id(&user.id) else {
            continue;
        };
        if mapping.get_user(&user.id).ok().flatten().is_some() {
            // Existing mapping; still emit membership because import is idempotent.
        }
        result.push(OrganizationMembership {
            user_id,
            organization_id: ctx.target_org_id,
            role: Role::Member,
            joined_at: ctx.now,
        });
    }
    result
}

fn transform_roles_and_permissions(
    ctx: &mut TransformContext,
    mapping: &MappingStore,
    roles: &[RocketRole],
) -> Result<(
    Vec<OrganizationRole>,
    Vec<Permission>,
    Vec<RuckRolePermission>,
)> {
    let mut org_roles = Vec::with_capacity(roles.len());
    let mut permissions = Vec::new();
    let mut grants = Vec::new();
    let mut seen_permissions: HashSet<String> = HashSet::new();

    for role in roles {
        let role_id = id_for("role", &role.id);

        let action = if mapping.get_role(&role.id)?.is_some() {
            "update"
        } else {
            "create"
        };
        mapping.put_role(&role.id, &role_id.to_string(), &role.name, action)?;

        org_roles.push(OrganizationRole {
            id: ruckchat_id::OrganizationRoleId::from_uuid(role_id),
            organization_id: ctx.target_org_id,
            name: role.name.clone(),
            description: role.description.clone(),
            created_at: ctx.now,
            updated_at: ctx.now,
        });

        for key in &role.permissions {
            let perm_id = id_for("permission", key);
            if seen_permissions.insert(key.to_string()) {
                permissions.push(Permission {
                    id: ruckchat_id::PermissionId::from_uuid(perm_id),
                    organization_id: ctx.target_org_id,
                    key: key.to_string(),
                    description: None,
                });
            }
            grants.push(RuckRolePermission {
                role_id: ruckchat_id::OrganizationRoleId::from_uuid(role_id),
                permission_id: ruckchat_id::PermissionId::from_uuid(perm_id),
            });
        }
    }

    Ok((org_roles, permissions, grants))
}

fn transform_teams(
    ctx: &mut TransformContext,
    mapping: &MappingStore,
    teams: &[RocketTeam],
) -> Result<Vec<Team>> {
    let mut result = Vec::with_capacity(teams.len());
    for team in teams {
        let team_id = id_for("team", &team.id);
        ctx.team_map.insert(team.id.clone(), team_id);

        let created_by = team
            .owner_id
            .as_deref()
            .and_then(|id| ctx.user_id(id))
            .unwrap_or(ctx.admin_user_id);

        let action = if mapping.get_team(team.id.clone()).ok().flatten().is_some() {
            "update"
        } else {
            "create"
        };
        mapping.put_team(team.id.clone(), team_id.to_string(), action)?;

        result.push(Team {
            id: ruckchat_id::TeamId::from_uuid(team_id),
            organization_id: ctx.target_org_id,
            name: team.name.clone(),
            description: team.description.clone(),
            created_by,
            created_at: parse_timestamp(team.created_at.as_deref().unwrap_or(""))
                .unwrap_or(ctx.now),
            updated_at: parse_timestamp(team.updated_at.as_deref().unwrap_or(""))
                .unwrap_or(ctx.now),
        });
    }
    Ok(result)
}

fn transform_team_memberships(
    ctx: &TransformContext,
    team_members: &HashMap<String, Vec<String>>,
) -> Vec<TeamMembership> {
    let mut result = Vec::new();
    for (team_rocket_id, usernames) in team_members {
        let Some(team_id) = ctx.team_id(team_rocket_id) else {
            continue;
        };
        for username in usernames {
            let Some(user_id) = ctx.user_by_username(username) else {
                continue;
            };
            result.push(TeamMembership {
                team_id: ruckchat_id::TeamId::from_uuid(team_id),
                user_id,
                role: TeamRole::Member,
                joined_at: ctx.now,
            });
        }
    }
    result
}

fn transform_rooms(
    ctx: &mut TransformContext,
    mapping: &MappingStore,
    rooms: &[RocketRoom],
) -> Result<(Vec<Channel>, Vec<DirectMessageConversation>)> {
    let mut channels = Vec::new();
    let mut dms = Vec::new();

    for room in rooms {
        if is_direct_message(&room.room_type) {
            let conversation_id = id_for("dm", &room.id);
            ctx.room_map.insert(room.id.clone(), conversation_id);

            let mut member_ids: Vec<UserId> = room
                .usernames
                .iter()
                .chain(room.users.iter())
                .filter_map(|u| ctx.user_by_username(u))
                .collect();
            member_ids.sort_unstable();
            member_ids.dedup();
            if member_ids.len() < 2 {
                member_ids.push(ctx.admin_user_id);
                member_ids.sort_unstable();
                member_ids.dedup();
            }
            if member_ids.len() >= 2 {
                let action = if mapping.get_room(&room.id)?.is_some() {
                    "update"
                } else {
                    "create"
                };
                mapping.put_room(
                    &room.id,
                    &conversation_id.to_string(),
                    &room.room_type,
                    "dm",
                    action,
                )?;

                dms.push(DirectMessageConversation {
                    id: DirectMessageConversationId::from_uuid(conversation_id),
                    organization_id: ctx.target_org_id,
                    member_ids,
                    created_at: parse_timestamp(room.ts.as_deref().unwrap_or(""))
                        .unwrap_or(ctx.now),
                });
            }
            continue;
        }

        let channel_id = id_for("channel", &room.id);
        ctx.room_map.insert(room.id.clone(), channel_id);

        let name = sanitize_channel_name(&room_display_name(room));
        let is_private = room.room_type == "p" || room.room_type == "v";
        let created_by = room
            .u
            .as_ref()
            .and_then(|u| ctx.user_id(&u.id))
            .unwrap_or(ctx.admin_user_id);
        let archived_at = if room.archived { Some(ctx.now) } else { None };

        let action = if mapping.get_room(&room.id)?.is_some() {
            "update"
        } else {
            "create"
        };
        mapping.put_room(
            &room.id,
            &channel_id.to_string(),
            &room.room_type,
            "channel",
            action,
        )?;

        channels.push(Channel {
            id: ChannelId::from_uuid(channel_id),
            organization_id: ctx.target_org_id,
            name,
            topic: room.topic.clone(),
            purpose: room.description.clone(),
            is_private,
            created_by,
            created_at: parse_timestamp(room.ts.as_deref().unwrap_or("")).unwrap_or(ctx.now),
            archived_at,
        });
    }

    Ok((channels, dms))
}

fn build_channel_memberships(
    ctx: &TransformContext,
    rooms: &[RocketRoom],
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

        let member_ids: Vec<UserId> = room
            .usernames
            .iter()
            .chain(room.users.iter())
            .filter_map(|u| ctx.user_by_username(u))
            .chain(room.u.as_ref().and_then(|u| ctx.user_id(&u.id)))
            .collect();

        for user_id in member_ids {
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

fn build_team_rooms(ctx: &TransformContext, rooms: &[RocketRoom]) -> Vec<TeamRoom> {
    let mut result = Vec::new();
    for room in rooms {
        let Some(team_rocket_id) = &room.team_id else {
            continue;
        };
        let Some(team_id) = ctx.team_id(team_rocket_id) else {
            continue;
        };
        let Some(channel_id) = ctx.room_id(&room.id) else {
            continue;
        };
        result.push(TeamRoom {
            team_id: ruckchat_id::TeamId::from_uuid(team_id),
            channel_id: ChannelId::from_uuid(channel_id),
            added_at: ctx.now,
        });
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
            let Some(author_id) = ctx.user_id(&msg.user.id) else {
                warn!(rocket_user_id = %msg.user.id, "message author not found; skipping");
                continue;
            };

            let message_id = MessageId::from_uuid(id_for("message", &msg.id));
            message_id_map.insert(msg.id.clone(), message_id);

            let parent_id = msg
                .tmid
                .as_ref()
                .and_then(|pid| message_id_map.get(pid).copied());

            let deleted_at = if msg.deleted_at.is_some() {
                Some(ctx.now)
            } else {
                None
            };
            if ctx.options.skip_deleted_messages && deleted_at.is_some() {
                continue;
            }

            let created_at = parse_timestamp(msg.ts.as_deref().unwrap_or("")).unwrap_or(ctx.now);
            let updated_at = msg
                .edited_at
                .as_deref()
                .and_then(parse_timestamp)
                .unwrap_or(created_at);

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
                created_at,
                updated_at,
                deleted_at,
            });

            for (emoji, value) in &msg.reactions {
                let usernames: Vec<String> = value
                    .get("usernames")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                for username in usernames {
                    let Some(user_id) = ctx.user_by_username(&username) else {
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
    fn parse_valid_timestamp() {
        let ts = parse_timestamp("2026-07-23T12:00:00Z");
        assert!(ts.is_some());
    }
}
