//! Integration tests for the RuckChat service layer.
//!
//! These tests exercise the SQLx-backed repositories and services against a live
//! PostgreSQL database. Run them with `DATABASE_URL` set to a migrated database:
//!
//! ```text
//! export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
//! cargo test -p ruckchat-server --test services
//! ```

use ruckchat_config::DatabaseConfig;
use ruckchat_domain::{ConversationType, Organization, Role, User, UserRepository};
use ruckchat_server::{
    connect_database,
    repositories::{
        ChannelMembershipRepositorySqlx, ChannelRepositorySqlx,
        DirectMessageConversationRepositorySqlx, FileRepositorySqlx, MessageRepositorySqlx,
        OrganizationMembershipRepositorySqlx, OrganizationRepositorySqlx,
        OrganizationSettingsRepositorySqlx, SessionRepositorySqlx, UserRepositorySqlx,
    },
    services::{
        AuthService, ChannelService, DirectMessageService, FileService, MessageService,
        OrganizationService, UserService,
        auth::AuthServiceDeps,
        authorization::AuthorizationService,
        channel::ChannelServiceDeps,
        direct_message::DirectMessageServiceDeps,
        dto::{
            AttachFileRequest, ChangeRoleRequest, CreateChannelRequest, CreateOrganizationRequest,
            EditMessageRequest, InviteMemberRequest, LoginRequest, PostMessageRequest,
            RecordUploadRequest, RegisterRequest, StartDmRequest, UpdateProfileRequest,
        },
        file::FileServiceDeps,
        message::MessageServiceDeps,
        organization::OrganizationServiceDeps,
        user::UserServiceDeps,
    },
};
use sqlx::PgPool;
use std::sync::Arc;

/// All services wired with SQLx repositories for a single test.
struct Services {
    #[allow(dead_code)]
    pool: PgPool,
    auth: AuthService,
    users: UserService,
    organizations: OrganizationService,
    channels: ChannelService,
    messages: MessageService,
    direct_messages: DirectMessageService,
    files: FileService,
    users_repo: Arc<dyn ruckchat_domain::UserRepository + Send + Sync>,
    memberships_repo: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync>,
}

async fn services() -> Services {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ruckchat:ruckchat@localhost/ruckchat".into());
    let config = DatabaseConfig::from_url(url);
    let pool = connect_database(&config)
        .await
        .expect("connect to postgres");

    let users: Arc<dyn ruckchat_domain::UserRepository + Send + Sync> =
        Arc::new(UserRepositorySqlx::new(pool.clone()));
    let sessions: Arc<dyn ruckchat_domain::SessionRepository + Send + Sync> =
        Arc::new(SessionRepositorySqlx::new(pool.clone()));
    let organizations: Arc<dyn ruckchat_domain::OrganizationRepository + Send + Sync> =
        Arc::new(OrganizationRepositorySqlx::new(pool.clone()));
    let memberships: Arc<dyn ruckchat_domain::OrganizationMembershipRepository + Send + Sync> =
        Arc::new(OrganizationMembershipRepositorySqlx::new(pool.clone()));
    let settings: Arc<dyn ruckchat_domain::OrganizationSettingsRepository + Send + Sync> =
        Arc::new(OrganizationSettingsRepositorySqlx::new(pool.clone()));
    let channels: Arc<dyn ruckchat_domain::ChannelRepository + Send + Sync> =
        Arc::new(ChannelRepositorySqlx::new(pool.clone()));
    let channel_memberships: Arc<dyn ruckchat_domain::ChannelMembershipRepository + Send + Sync> =
        Arc::new(ChannelMembershipRepositorySqlx::new(pool.clone()));
    let messages_repo: Arc<dyn ruckchat_domain::MessageRepository + Send + Sync> =
        Arc::new(MessageRepositorySqlx::new(pool.clone()));
    let conversations: Arc<dyn ruckchat_domain::DirectMessageConversationRepository + Send + Sync> =
        Arc::new(DirectMessageConversationRepositorySqlx::new(pool.clone()));
    let files_repo: Arc<dyn ruckchat_domain::FileRepository + Send + Sync> =
        Arc::new(FileRepositorySqlx::new(pool.clone()));

    let authz = AuthorizationService::new();
    let messages_for_files = messages_repo.clone();

    Services {
        auth: AuthService::new(AuthServiceDeps {
            users: users.clone(),
            sessions,
            organizations: organizations.clone(),
            memberships: memberships.clone(),
            settings: settings.clone(),
            channels: channels.clone(),
            channel_memberships: channel_memberships.clone(),
        }),
        users: UserService::new(UserServiceDeps {
            users: users.clone(),
            memberships: memberships.clone(),
        }),
        organizations: OrganizationService::new(OrganizationServiceDeps {
            organizations: organizations.clone(),
            users: users.clone(),
            memberships: memberships.clone(),
            settings: settings.clone(),
            authorization: authz.clone(),
        }),
        channels: ChannelService::new(ChannelServiceDeps {
            channels: channels.clone(),
            channel_memberships: channel_memberships.clone(),
            memberships: memberships.clone(),
            authorization: authz.clone(),
        }),
        messages: MessageService::new(MessageServiceDeps {
            messages: messages_repo,
            channels: channels.clone(),
            channel_memberships: channel_memberships.clone(),
            memberships: memberships.clone(),
            conversations: conversations.clone(),
            authorization: authz.clone(),
        }),
        direct_messages: DirectMessageService::new(DirectMessageServiceDeps {
            conversations: conversations.clone(),
            memberships: memberships.clone(),
        }),
        files: FileService::new(FileServiceDeps {
            files: files_repo,
            messages: messages_for_files,
            memberships: memberships.clone(),
            settings: settings.clone(),
        }),
        users_repo: users.clone(),
        memberships_repo: memberships.clone(),
        pool,
    }
}

fn unique(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4())
}

fn unique_email(prefix: &str) -> String {
    format!("{}+{}@example.com", prefix, uuid::Uuid::new_v4())
}

async fn seed_user(repo: &Arc<dyn UserRepository + Send + Sync>, prefix: &str) -> User {
    let user = User::new(unique_email(prefix), "Test User", "hash").unwrap();
    repo.create(&user).await.unwrap();
    user
}

async fn register_user(auth: &AuthService, prefix: &str) -> (User, Organization) {
    let req = RegisterRequest {
        email: unique_email(prefix),
        display_name: "Owner".into(),
        password: "correcthorsebatterystaple".into(),
        organization_name: "Acme".into(),
        organization_slug: unique("acme"),
    };
    auth.register(req).await.unwrap()
}

#[tokio::test]
async fn auth_register_login_and_authenticate() {
    let svc = services().await;
    let (user, _org) = register_user(&svc.auth, "auth").await;

    let login = svc
        .auth
        .login(LoginRequest {
            email: user.email.clone(),
            password: "correcthorsebatterystaple".into(),
        })
        .await
        .unwrap();
    assert_eq!(login.user_id, user.id);

    let auth_id = svc.auth.authenticate(&login.token).await.unwrap();
    assert_eq!(auth_id, user.id);
}

#[tokio::test]
async fn organization_lifecycle() {
    let svc = services().await;
    let (owner, initial_org) = register_user(&svc.auth, "orgowner").await;

    let org2 = svc
        .organizations
        .create_organization(
            owner.id,
            CreateOrganizationRequest {
                name: "Beta".into(),
                slug: unique("beta"),
            },
        )
        .await
        .unwrap();

    let member = seed_user(&svc.users_repo, "member").await;
    svc.organizations
        .invite_member(
            owner.id,
            org2.id,
            InviteMemberRequest {
                email: member.email.clone(),
                role: Role::Member,
            },
        )
        .await
        .unwrap();

    let membership = svc
        .memberships_repo
        .by_ids(member.id, org2.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(membership.role, Role::Member);

    svc.organizations
        .change_member_role(
            owner.id,
            org2.id,
            ChangeRoleRequest {
                user_id: member.id,
                role: Role::Admin,
            },
        )
        .await
        .unwrap();
    let membership = svc
        .memberships_repo
        .by_ids(member.id, org2.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(membership.role, Role::Admin);

    let list = svc.organizations.list_for_user(owner.id).await.unwrap();
    assert!(list.iter().any(|o| o.id == initial_org.id));
    assert!(list.iter().any(|o| o.id == org2.id));
}

#[tokio::test]
async fn channel_visibility_and_membership() {
    let svc = services().await;
    let (owner, _) = register_user(&svc.auth, "chanowner").await;

    let org = svc
        .organizations
        .create_organization(
            owner.id,
            CreateOrganizationRequest {
                name: "Team".into(),
                slug: unique("team"),
            },
        )
        .await
        .unwrap();

    let public = svc
        .channels
        .create_channel(
            owner.id,
            org.id,
            CreateChannelRequest {
                name: "general".into(),
                is_private: false,
            },
        )
        .await
        .unwrap();
    let private = svc
        .channels
        .create_channel(
            owner.id,
            org.id,
            CreateChannelRequest {
                name: "secret".into(),
                is_private: true,
            },
        )
        .await
        .unwrap();

    let member = seed_user(&svc.users_repo, "chanmember").await;
    svc.organizations
        .invite_member(
            owner.id,
            org.id,
            InviteMemberRequest {
                email: member.email.clone(),
                role: Role::Member,
            },
        )
        .await
        .unwrap();

    let visible = svc
        .channels
        .list_channels_in_organization(member.id, org.id)
        .await
        .unwrap();
    assert!(visible.iter().any(|c| c.id == public.id));
    assert!(!visible.iter().any(|c| c.id == private.id));

    svc.channels
        .add_member(owner.id, private.id, member.id)
        .await
        .unwrap();
    let visible = svc
        .channels
        .list_channels_in_organization(member.id, org.id)
        .await
        .unwrap();
    assert!(visible.iter().any(|c| c.id == private.id));
}

#[tokio::test]
async fn message_post_edit_delete_and_history() {
    let svc = services().await;
    let (owner, org) = register_user(&svc.auth, "msgowner").await;

    let general = svc
        .channels
        .list_channels_in_organization(owner.id, org.id)
        .await
        .unwrap()
        .pop()
        .unwrap();

    let msg = svc
        .messages
        .post_message(
            owner.id,
            PostMessageRequest {
                conversation_id: general.id.as_uuid(),
                conversation_type: ConversationType::Channel,
                parent_id: None,
                content: "hello".into(),
            },
        )
        .await
        .unwrap();

    let history = svc
        .messages
        .get_history(
            owner.id,
            general.id.as_uuid(),
            ConversationType::Channel,
            Default::default(),
        )
        .await
        .unwrap();
    assert!(history.iter().any(|m| m.id == msg.id));

    let edited = svc
        .messages
        .edit_message(
            owner.id,
            msg.id,
            EditMessageRequest {
                content: "hello world".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(edited.content, "hello world");

    svc.messages.delete_message(owner.id, msg.id).await.unwrap();
    let history = svc
        .messages
        .get_history(
            owner.id,
            general.id.as_uuid(),
            ConversationType::Channel,
            Default::default(),
        )
        .await
        .unwrap();
    assert!(!history.iter().any(|m| m.id == msg.id));
}

#[tokio::test]
async fn direct_message_flow() {
    let svc = services().await;
    let (a, org) = register_user(&svc.auth, "dma").await;
    let b = seed_user(&svc.users_repo, "dmb").await;

    svc.organizations
        .invite_member(
            a.id,
            org.id,
            InviteMemberRequest {
                email: b.email.clone(),
                role: Role::Member,
            },
        )
        .await
        .unwrap();

    let dm = svc
        .direct_messages
        .start_conversation(
            a.id,
            StartDmRequest {
                organization_id: org.id,
                member_ids: vec![b.id],
            },
        )
        .await
        .unwrap();

    let msg = svc
        .messages
        .post_message(
            a.id,
            PostMessageRequest {
                conversation_id: dm.id.as_uuid(),
                conversation_type: ConversationType::DirectMessage,
                parent_id: None,
                content: "dm hello".into(),
            },
        )
        .await
        .unwrap();

    let dms = svc
        .direct_messages
        .list_conversations_for_user(b.id, org.id)
        .await
        .unwrap();
    assert!(dms.iter().any(|c| c.id == dm.id));

    let history = svc
        .messages
        .get_history(
            b.id,
            dm.id.as_uuid(),
            ConversationType::DirectMessage,
            Default::default(),
        )
        .await
        .unwrap();
    assert!(history.iter().any(|m| m.id == msg.id));
}

#[tokio::test]
async fn file_upload_attach_and_list() {
    let svc = services().await;
    let (owner, org) = register_user(&svc.auth, "fileowner").await;

    let file = svc
        .files
        .record_upload(
            owner.id,
            RecordUploadRequest {
                organization_id: org.id,
                file_name: "report.pdf".into(),
                mime_type: "application/pdf".into(),
                size_bytes: 1024,
                storage_path: "s3://bucket/report.pdf".into(),
            },
        )
        .await
        .unwrap();

    let meta = svc.files.get_file_metadata(file.id).await.unwrap();
    assert_eq!(meta.file_name, "report.pdf");

    let general = svc
        .channels
        .list_channels_in_organization(owner.id, org.id)
        .await
        .unwrap()
        .pop()
        .unwrap();
    let msg = svc
        .messages
        .post_message(
            owner.id,
            PostMessageRequest {
                conversation_id: general.id.as_uuid(),
                conversation_type: ConversationType::Channel,
                parent_id: None,
                content: "see attachment".into(),
            },
        )
        .await
        .unwrap();

    svc.files
        .attach_file_to_message(
            owner.id,
            AttachFileRequest {
                message_id: msg.id,
                file_id: file.id,
            },
        )
        .await
        .unwrap();

    let list = svc
        .files
        .list_files_in_organization(owner.id, org.id)
        .await
        .unwrap();
    assert!(list.iter().any(|f| f.id == file.id));
}

#[tokio::test]
async fn user_profile_update() {
    let svc = services().await;
    let (user, _org) = register_user(&svc.auth, "profile").await;

    let updated = svc
        .users
        .update_profile(
            user.id,
            UpdateProfileRequest {
                display_name: Some("New Name".into()),
                avatar_url: Some("https://example.com/avatar.png".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.display_name, "New Name");
    assert_eq!(
        updated.avatar_url,
        Some("https://example.com/avatar.png".into())
    );

    let profile = svc.users.get_profile(user.id).await.unwrap();
    assert_eq!(profile.display_name, "New Name");
}
