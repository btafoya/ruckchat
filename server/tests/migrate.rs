//! Integration tests for the data migration export/import commands.

use ruckchat_domain::{
    Channel, ChannelMembership, DirectMessageConversation, File, Message, Organization,
    OrganizationMembership, OrganizationSettings, Reaction, User,
};
use ruckchat_domain::{
    ChannelMembershipRepository, ChannelRepository, DirectMessageConversationRepository,
    FileRepository, MessageRepository, OrganizationMembershipRepository, OrganizationRepository,
    OrganizationSettingsRepository, ReactionRepository, UserRepository,
};
use ruckchat_server::migrate::{self, ImportCounts, MigrationData};
use ruckchat_server::repositories::{
    ChannelMembershipRepositorySqlx, ChannelRepositorySqlx,
    DirectMessageConversationRepositorySqlx, FileRepositorySqlx, MessageRepositorySqlx,
    OrganizationMembershipRepositorySqlx, OrganizationRepositorySqlx,
    OrganizationSettingsRepositorySqlx, ReactionRepositorySqlx, UserRepositorySqlx,
};
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

#[sqlx::test]
async fn export_includes_all_domain_tables(pool: PgPool) {
    let data = seed_data(&pool).await;

    let exported = migrate::export(&pool).await.expect("export succeeds");

    assert_eq!(exported.version, 2);
    assert_eq!(exported.users.len(), 2);
    assert_eq!(exported.organizations.len(), 1);
    assert_eq!(exported.organization_memberships.len(), 2);
    assert_eq!(exported.organization_settings.len(), 1);
    assert_eq!(exported.organization_roles.len(), 0);
    assert_eq!(exported.permissions.len(), 0);
    assert_eq!(exported.role_permissions.len(), 0);
    assert_eq!(exported.custom_emoji.len(), 0);
    assert_eq!(exported.teams.len(), 0);
    assert_eq!(exported.team_memberships.len(), 0);
    assert_eq!(exported.team_rooms.len(), 0);
    assert_eq!(exported.channels.len(), 1);
    assert_eq!(exported.channel_memberships.len(), 2);
    assert_eq!(exported.direct_message_conversations.len(), 1);
    assert_eq!(exported.messages.len(), 2);
    assert_eq!(exported.reactions.len(), 1);
    assert_eq!(exported.files.len(), 1);
    assert_eq!(exported.message_files.len(), 1);

    let exported_user_ids: HashSet<Uuid> = exported.users.iter().map(|u| u.id.as_uuid()).collect();
    assert!(exported_user_ids.contains(&data.owner.id.as_uuid()));
    assert!(exported_user_ids.contains(&data.member.id.as_uuid()));
}

#[sqlx::test]
async fn import_is_idempotent(pool: PgPool) {
    seed_data(&pool).await;
    let snapshot = migrate::export(&pool).await.expect("export succeeds");

    let counts = migrate::import(&pool, &snapshot, false)
        .await
        .expect("import succeeds");

    assert_eq!(
        counts,
        ImportCounts {
            inserted: 0,
            skipped: total_rows(&snapshot),
        }
    );
}

#[sqlx::test]
async fn round_trip_export_import_export_matches(pool: PgPool) {
    let _ = seed_data(&pool).await;
    let before = migrate::export(&pool).await.expect("first export");
    let temp_path = std::env::temp_dir().join(format!("ruckchat-migrate-{}.json", Uuid::new_v4()));
    migrate::export_to_file(&pool, &temp_path)
        .await
        .expect("export to file");

    // Clear all exportable tables so the import can re-insert every row.
    sqlx::query!(
        "TRUNCATE TABLE users, organizations, organization_memberships, organization_settings,
         organization_roles, permissions, organization_role_permissions, custom_emoji,
         teams, team_memberships, team_rooms, channels, channel_memberships,
         direct_message_conversations, dm_members, messages, reactions, files, message_files
         RESTART IDENTITY CASCADE"
    )
    .execute(&pool)
    .await
    .unwrap();

    let from_file = migrate::read_migration_file(&temp_path)
        .await
        .expect("read migration file");
    let counts = migrate::import(&pool, &from_file, false)
        .await
        .expect("import after truncate");
    assert_eq!(counts.inserted, total_rows(&from_file));
    assert_eq!(counts.skipped, 0);

    let after = migrate::export(&pool).await.expect("second export");
    assert_eq!(before.users.len(), after.users.len());
    assert_eq!(before.organizations.len(), after.organizations.len());
    assert_eq!(before.channels.len(), after.channels.len());
    assert_eq!(before.messages.len(), after.messages.len());

    tokio::fs::remove_file(&temp_path).await.ok();
}

#[sqlx::test]
async fn dry_run_does_not_write(pool: PgPool) {
    seed_data(&pool).await;
    let snapshot = migrate::export(&pool).await.expect("export");

    // Clear all exportable tables and dry-run import.
    sqlx::query!(
        "TRUNCATE TABLE users, organizations, organization_memberships, organization_settings,
         organization_roles, permissions, organization_role_permissions, custom_emoji,
         teams, team_memberships, team_rooms, channels, channel_memberships,
         direct_message_conversations, dm_members, messages, reactions, files, message_files
         RESTART IDENTITY CASCADE"
    )
    .execute(&pool)
    .await
    .unwrap();

    let counts = migrate::import(&pool, &snapshot, true)
        .await
        .expect("dry run import");
    assert_eq!(counts.inserted, 0);
    assert_eq!(counts.skipped, total_rows(&snapshot));

    let after = migrate::export(&pool).await.expect("export after dry run");
    assert!(after.users.is_empty());
}

#[derive(Debug)]
#[allow(dead_code)]
struct SeedData {
    owner: User,
    member: User,
    organization: Organization,
    channel: Channel,
    dm: DirectMessageConversation,
    message: Message,
    reply: Message,
    file: File,
}

async fn seed_data(pool: &PgPool) -> SeedData {
    ruckchat_migrations::migrator()
        .run(pool)
        .await
        .expect("migrations apply");

    let users = UserRepositorySqlx::new(pool.clone());
    let organizations = OrganizationRepositorySqlx::new(pool.clone());
    let memberships = OrganizationMembershipRepositorySqlx::new(pool.clone());
    let settings = OrganizationSettingsRepositorySqlx::new(pool.clone());
    let channels = ChannelRepositorySqlx::new(pool.clone());
    let channel_memberships = ChannelMembershipRepositorySqlx::new(pool.clone());
    let conversations = DirectMessageConversationRepositorySqlx::new(pool.clone());
    let messages = MessageRepositorySqlx::new(pool.clone());
    let reactions = ReactionRepositorySqlx::new(pool.clone());
    let files = FileRepositorySqlx::new(pool.clone());

    let owner = User::new("owner@example.com", "Owner", "owner-hash").unwrap();
    let member = User::new("member@example.com", "Member", "member-hash").unwrap();
    users.create(&owner).await.unwrap();
    users.create(&member).await.unwrap();

    let organization = Organization::new("Acme", "acme", owner.id).unwrap();
    organizations.create(&organization).await.unwrap();

    let owner_membership =
        OrganizationMembership::new(owner.id, organization.id, ruckchat_domain::Role::Owner)
            .unwrap();
    let member_membership =
        OrganizationMembership::new(member.id, organization.id, ruckchat_domain::Role::Member)
            .unwrap();
    memberships.create(&owner_membership).await.unwrap();
    memberships.create(&member_membership).await.unwrap();

    let org_settings = OrganizationSettings::new(organization.id);
    settings.create(&org_settings).await.unwrap();

    let mut channel = Channel::new(organization.id, "general", owner.id, false).unwrap();
    channel.set_topic(Some("General chat"));
    channels.create(&channel).await.unwrap();

    channel_memberships
        .create(&ChannelMembership::new(owner.id, channel.id).unwrap())
        .await
        .unwrap();
    channel_memberships
        .create(&ChannelMembership::new(member.id, channel.id).unwrap())
        .await
        .unwrap();

    let dm = DirectMessageConversation::new(organization.id, vec![owner.id, member.id]).unwrap();
    conversations.create(&dm).await.unwrap();

    let message = Message::new(
        channel.id.as_uuid(),
        ruckchat_domain::ConversationType::Channel,
        owner.id,
        "Hello world",
        None,
        vec![],
    )
    .unwrap();
    messages.create(&message).await.unwrap();

    let reply = Message::new(
        channel.id.as_uuid(),
        ruckchat_domain::ConversationType::Channel,
        member.id,
        "Reply",
        Some(message.id),
        vec![],
    )
    .unwrap();
    messages.create(&reply).await.unwrap();

    let reaction = Reaction::new(message.id, member.id, "👋").unwrap();
    reactions.create(&reaction).await.unwrap();

    let file = File::new(
        organization.id,
        owner.id,
        "report.pdf",
        "application/pdf",
        1024,
        "files/report.pdf",
    )
    .unwrap();
    files.create(&file).await.unwrap();
    files.attach_to_message(message.id, file.id).await.unwrap();

    SeedData {
        owner,
        member,
        organization,
        channel,
        dm,
        message,
        reply,
        file,
    }
}

fn total_rows(data: &MigrationData) -> usize {
    data.users.len()
        + data.organizations.len()
        + data.organization_memberships.len()
        + data.organization_settings.len()
        + data.organization_roles.len()
        + data.permissions.len()
        + data.role_permissions.len()
        + data.custom_emoji.len()
        + data.teams.len()
        + data.team_memberships.len()
        + data.team_rooms.len()
        + data.channels.len()
        + data.channel_memberships.len()
        + data.direct_message_conversations.len()
        + data.messages.len()
        + data.reactions.len()
        + data.files.len()
        + data.message_files.len()
}
