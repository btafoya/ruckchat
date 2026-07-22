# 004 - Domain

The domain model lives in the shared `ruckchat-domain` crate. It contains the
core entities, value objects, and repository interfaces used by the server
service layer and later by clients and plugins.

## Crate Layout

```
crates/ruckchat-domain/src
├── lib.rs                         # Module re-exports
├── user.rs
├── organization.rs
├── organization_membership.rs
├── organization_settings.rs
├── channel.rs
├── channel_membership.rs
├── direct_message_conversation.rs
├── message.rs
├── reaction.rs
├── file.rs
├── session.rs
├── role.rs
└── repositories.rs                # Async repository traits
```

## Core Entities

### User

- Represents a human account.
- Fields: `id`, `email`, `display_name`, `password_hash`, `avatar_url`, `created_at`, `updated_at`.
- A user can belong to multiple organizations.
- Email addresses are globally unique.
- `User::new` validates email format, display name length, and non-empty password hash.

### Organization

- Represents a tenant workspace.
- Fields: `id`, `name`, `slug`, `owner_id`, `created_at`, `updated_at`.
- Slug is used in URLs and must be unique and URL-safe.
- Deleting an organization deletes all contained channels, messages, and memberships.
- `Organization::new` validates name and slug format/length.

### Organization Membership

- Links a user to an organization with a role.
- Fields: `user_id`, `organization_id`, `role`, `joined_at`.
- Roles: `owner`, `admin`, `member`.
- A user has exactly one membership per organization.

### Channel

- A conversation space within an organization.
- Fields: `id`, `organization_id`, `name`, `topic`, `purpose`, `is_private`, `created_by`, `created_at`, `archived_at`.
- Channel names are unique within an organization and limited to lowercase letters, numbers, and hyphens.
- Public channels are readable by all organization members.
- Private channels are readable only by explicit members.
- `Channel::new` validates the channel name and supports `archive`/`unarchive`.

### Channel Membership

- Links a user to a channel.
- Fields: `user_id`, `channel_id`, `joined_at`.
- Membership is required to post or receive messages in private channels.
- Public channels do not require membership for reading, but membership is required for posting.

### Direct Message Conversation

- A conversation between two or more users.
- Fields: `id`, `organization_id`, `created_at`.
- Members are stored in a separate `dm_members` table.
- DM membership cannot change after creation in v1.
- `DirectMessageConversation::new` rejects duplicate members and requires at least two members.

### Message

- A single communication unit.
- Fields: `id`, `conversation_id` (channel or DM), `parent_id` (for threads), `author_id`, `content`, `created_at`, `updated_at`, `deleted_at`.
- Content is plain text with optional markdown formatting.
- Editing updates `content` and `updated_at` but never changes the conversation.
- Deletion sets `deleted_at` and clears the content; the record remains for history consistency.

### Reaction

- An emoji reaction to a message.
- Fields: `message_id`, `user_id`, `emoji`, `created_at`.
- Composite primary key on `(message_id, user_id, emoji)`.
- A user can only add one of each emoji to a message.

### File Attachment

- A file uploaded and attached to a message or conversation.
- Fields: `id`, `organization_id`, `uploaded_by`, `file_name`, `mime_type`, `size_bytes`, `storage_path`, `thumbnail_path`, `created_at`.
- The database stores metadata; file bytes live on the configured storage backend.

### Session

- An authenticated browser or client session.
- Fields: `id`, `user_id`, `token_hash`, `expires_at`, `created_at`, `ip_address`, `user_agent`.
- Sessions expire after a configurable idle period (default 30 days).
- Sessions are invalidated on password change or explicit logout.
- `Session::new` requires a future expiration and non-empty token hash.

## Value Objects

### `Role`

- `owner`, `admin`, `member`.
- `Role::is_manager` and `Role::is_moderator` helpers reflect admin/owner privileges.

### `ConversationType`

- `channel` or `dm`.
- Used by `Message` to disambiguate `conversation_id`.

## Domain Invariants

1. A user must be a member of an organization to access its channels or DMs.
2. A user must be a member of a private channel to read or post messages in it.
3. A user can only edit or delete their own messages unless they are an admin or owner.
4. An organization must always have at least one owner.
5. A channel name is unique within its organization and cannot be changed to conflict with another channel.
6. A DM cannot have duplicate members.
7. Reactions are unique per user per emoji per message.
8. File uploads are attributed to an organization and a user; storage quotas are enforced at the organization level.

## Aggregates and Ownership

- **Organization** is the top-level aggregate.
- **Channel** and **Direct Message Conversation** belong to an organization.
- **Message** and **Reaction** belong to a conversation.
- **File Attachment** belongs to an organization but may be linked to multiple messages through a join table if needed later.

## Repository Interfaces

Repository traits are defined in `crates/ruckchat-domain/src/repositories.rs` using
`async-trait`. They provide the boundary that the server service layer will use.
SQLx implementations live in the server crate. One trait exists per aggregate:

- `UserRepository`
- `SessionRepository`
- `OrganizationRepository`
- `OrganizationMembershipRepository`
- `OrganizationSettingsRepository`
- `ChannelRepository`
- `ChannelMembershipRepository`
- `DirectMessageConversationRepository`
- `MessageRepository`
- `ReactionRepository`
- `FileRepository`

## Validations

- Email: valid format, unique (enforced by repository and schema).
- Display name: 1-100 characters.
- Organization slug: 3-63 characters, lowercase letters, numbers, hyphens.
- Channel name: 1-80 characters, lowercase letters, numbers, hyphens.
- Message content: 1-4000 characters (configurable).
- File size: configurable per organization, default 25 MB per file.

Validation helpers reused from `ruckchat-common` include `validate_email`,
`validate_display_name`, `validate_slug`, and `validate_channel_name`.
