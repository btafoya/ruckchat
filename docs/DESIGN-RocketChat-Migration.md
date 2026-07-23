# Design: RocketChat → RuckChat Migration Tool

## Scope

Build a standalone, API-driven migration tool (`rocketchat2ruckchat`) that copies a complete RocketChat workspace into a target RuckChat organization. Because RuckChat v1 does not expose every concept required for a faithful migration, the design extends the server with admin import endpoints rather than touching the database directly.

## Goals

1. Migrate users, rooms (channels/groups/DMs/teams/discussions), messages, reactions, files, roles, permissions, and custom emoji from RocketChat to RuckChat.
2. Keep the migration tool standalone and REST-API-only.
3. Support re-runs: idempotent updates, resumption, and a persistent source→target ID mapping.
4. Default to dry-run; writes require `--apply`.
5. Provide interactive prompts for credentials, inaccessible rooms, and conflicts.

## Non-goals

- Real-time synchronization or continuous bridge mode.
- Migrating ephemeral runtime state (sessions, Web Push subscriptions, plugin state).
- Converting RocketChat apps, integrations, or Livechat/Omnichannel data.
- Modifying RuckChat schema outside normal migrations and documented endpoints.

## High-level architecture

```text
┌─────────────────────────────────────────────────────────────────────┐
│                         rocketchat2ruckchat                         │
│  (standalone Rust binary, interactive or config-driven, dry-run)    │
└──────────────┬──────────────────────────────────────┬───────────────┘
               │ reads                                │ writes
               ▼                                      ▼
┌──────────────────────────┐              ┌────────────────────────────┐
│   RocketChat REST API    │              │   RuckChat REST API          │
│   (PAT or user/password) │              │   (session cookie)           │
└──────────────┬───────────┘              └──────────────┬─────────────┘
               │                                          │
               │  SQLite mapping table                     │
               │  rocket_id → ruckchat_id                 │
               │                                          │
               ▼                                          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      RuckChat server extensions                     │
│  Admin import endpoints that accept a normalized snapshot and       │
│  write it idempotently through the existing service/repository layer. │
└─────────────────────────────────────────────────────────────────────┘
```

## Key design decision: snapshot import endpoint

Instead of adding dozens of individual admin CRUD endpoints for roles, permissions, emoji, teams, historical messages, and password hashes, the server exposes a single authenticated admin endpoint:

```
POST /api/v1/admin/organizations/:id/import
```

It accepts a versioned `MigrationData` JSON payload that mirrors the internal export format in `server/src/migrate.rs`, extended with the missing categories. The server validates the caller has `Owner` or `Admin` role on the target organization and writes the snapshot idempotently through the existing service and repository layer. This reuses the existing transaction-based import logic and avoids duplicating validation rules in the standalone tool.

The standalone tool’s main job is therefore:

1. Read RocketChat via its REST API.
2. Normalize RocketChat concepts into RuckChat `MigrationData`.
3. Optionally download and re-upload file bytes.
4. Call `POST /api/v1/admin/organizations/:id/import` with `--apply`, or report the computed snapshot in dry-run mode.

## Phase sequence

This work is too large for a single implementation pass. Sequence it as:

1. **Phase A — Extend `MigrationData` and import endpoint**
   - Add database migrations for roles, permissions, custom emoji, teams, and team memberships.
   - Add domain entities, repository traits, services, and handlers.
   - Extend `server/src/migrate.rs` `MigrationData` and import logic to cover the new categories.
   - Add `POST /api/v1/admin/organizations/:id/import` and supporting list endpoints.
   - Update OpenAPI.

2. **Phase B — Build `rocketchat2ruckchat`**
   - Create a new Rust binary crate outside the main workspace or as a workspace member that does not depend on server internals.
   - Implement RocketChat client, RuckChat client, mapping store, pipeline, dry-run, and interactive prompts.

## Phase A: RuckChat server extensions

### Database migrations

#### 1. Custom organization roles

```sql
CREATE TABLE organization_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);
```

#### 2. Permission matrix

RuckChat currently hard-codes permissions in `server/src/services/authorization.rs`. For migration we need a stored mapping so custom RocketChat roles can be preserved.

```sql
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    key TEXT NOT NULL,          -- e.g. 'manage_channels'
    description TEXT,
    UNIQUE (organization_id, key)
);

CREATE TABLE organization_role_permissions (
    role_id UUID NOT NULL REFERENCES organization_roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
```

Seed the built-in `owner`/`admin`/`member` roles as implicit rows, or treat them as special-cased defaults when no custom role exists.

#### 3. Custom emoji

```sql
CREATE TABLE custom_emoji (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    shortcode TEXT NOT NULL,    -- without colons, e.g. 'partyparrot'
    file_id UUID NOT NULL REFERENCES files(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, shortcode)
);
```

#### 4. Teams

```sql
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);

CREATE TABLE team_memberships (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'leader', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

CREATE TABLE team_rooms (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, channel_id)
);
```

#### 5. User `active` / `deactivated_at`

RocketChat deleted users should become deactivated, not removed. Add a nullable `deactivated_at` column to `users` (or `is_active` boolean) and expose it in the domain model.

```sql
ALTER TABLE users ADD COLUMN deactivated_at TIMESTAMPTZ;
```

### Domain entities (`ruckchat-domain`)

Add new aggregates:

- `OrganizationRole` — id, organization_id, name, description.
- `Permission` — id, organization_id, key, description.
- `OrganizationRolePermission` — role_id, permission_id.
- `CustomEmoji` — id, organization_id, shortcode, file_id, created_by.
- `Team` — id, organization_id, name, description, created_by.
- `TeamMembership` — team_id, user_id, role, joined_at.
- `TeamRoom` — team_id, channel_id, added_at.

Extend existing aggregates:

- `User` — add `deactivated_at: Option<OffsetDateTime>`.
- `Channel` — already has `archived_at` for archived rooms.
- `Message` — already has `deleted_at` for deleted messages.

Add repository traits in `crates/ruckchat-domain/src/repositories.rs` for each new aggregate.

### Service layer (`server/src/services`)

Create `AdminService` (or extend `OrganizationService`) that:

- Verifies the caller is an `Owner` or `Admin` of the target organization.
- Validates the incoming `MigrationData` snapshot.
- Writes users, org settings, roles, permissions, emoji, teams, channels, memberships, DMs, messages, reactions, files, and message-file links idempotently inside a SQLx transaction.
- For users: create or update by email; if a pre-hashed password is provided, store it verbatim (requires a new admin-only create-user path that bypasses normal hashing).
- For messages: create or update by ID, preserving original `created_at`, `updated_at`, `deleted_at`, `parent_id`, and `author_id`. This bypasses normal authorship/time validation and is restricted to the admin import endpoint.

### REST handlers

New admin routes under `/api/v1/admin`:

| Method | Path | Purpose |
|--------|------|---------|
| POST   | `/admin/organizations/:id/import` | Import a `MigrationData` snapshot |
| GET    | `/admin/organizations/:id/import/status/:job_id` | (optional) async import progress |
| GET    | `/admin/organizations/:id/roles` | List custom roles |
| POST   | `/admin/organizations/:id/roles` | Create custom role |
| GET    | `/admin/organizations/:id/permissions` | List permissions |
| GET    | `/admin/organizations/:id/emoji` | List custom emoji |
| POST   | `/admin/organizations/:id/emoji` | Create custom emoji |
| GET    | `/admin/organizations/:id/teams` | List teams |
| POST   | `/admin/organizations/:id/teams` | Create team |

The migration tool only needs the import endpoint. The others are exposed for day-to-day admin UI parity.

### `MigrationData` extension

Extend the existing `server/src/migrate.rs` `MigrationData` struct:

```rust
pub struct MigrationData {
    pub version: u16,                         // bump to 2
    pub exported_at: OffsetDateTime,
    pub users: Vec<User>,                     // extended with deactivated_at
    pub organizations: Vec<Organization>,
    pub organization_memberships: Vec<OrganizationMembership>,
    pub organization_settings: Vec<OrganizationSettings>,
    pub organization_roles: Vec<OrganizationRole>,
    pub permissions: Vec<Permission>,
    pub role_permissions: Vec<OrganizationRolePermission>,
    pub custom_emoji: Vec<CustomEmoji>,
    pub teams: Vec<Team>,
    pub team_memberships: Vec<TeamMembership>,
    pub team_rooms: Vec<TeamRoom>,
    pub channels: Vec<Channel>,
    pub channel_memberships: Vec<ChannelMembership>,
    pub direct_message_conversations: Vec<DirectMessageConversation>,
    pub messages: Vec<MigrationMessage>,
    pub reactions: Vec<Reaction>,
    pub files: Vec<File>,
    pub message_files: Vec<MessageFileLink>,
}
```

Version 1 readers can ignore the new fields. The import endpoint accepts both versions and maps them into the new schema.

## Phase B: `rocketchat2ruckchat` standalone tool

### Crate structure

Create `tools/rocketchat2ruckchat/` or `crates/rocketchat2ruckchat/` as a workspace member. It must not import `ruckchat-server` internals; it may import `ruckchat-common` and `ruckchat-domain` only for shared DTO types if those crates remain dependency-light. If that creates coupling, define local DTOs that mirror `MigrationData`.

```text
crates/rocketchat2ruckchat/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs          // CLI + YAML + interactive prompts
    ├── rocket_chat/
    │   ├── client.rs      // reqwest wrapper
    │   ├── auth.rs
    │   ├── models.rs      // RocketChat response structs
    │   └── pagination.rs
    ├── ruckchat/
    │   ├── client.rs      // reqwest wrapper with cookie jar
    │   ├── auth.rs
    │   └── models.rs      // RuckChat request/response structs
    ├── mapping.rs         // SQLite mapping store
    ├── pipeline.rs        // ordered migration stages
    ├── transform.rs       // RocketChat → MigrationData
    ├── dry_run.rs         // report generation
    ├── interactive.rs     // prompt helpers
    └── report.rs
```

### Configuration

```yaml
source:
  url: https://rocketchat.example.com
  auth:
    pat:
      user_id: abc123
      auth_token: xyz789
    # OR
    # login:
    #   username: admin
    #   password: ""

target:
  url: http://localhost:3000
  auth:
    login:
      email: admin@example.com
      password: ""
  organization_id: 00000000-0000-0000-0000-000000000000

options:
  scope:
    - users
    - rooms
    - messages
    - reactions
    - files
    - roles
    - permissions
    - emoji
    - teams
  map_existing_users: true
  deactivate_deleted_users: true
  archive_deleted_rooms: true
  skip_deleted_messages: true
  dry_run: true

mapping_store: ./rocketchat2ruckchat.mapping.sqlite
```

Missing values trigger interactive prompts.

### Authentication

- **RocketChat**: try PAT first; if login credentials are supplied, call `POST /api/v1/login` and use the returned `authToken`/`userId`.
- **RuckChat**: call `POST /api/v1/auth/login`, store the `ruckchat_session` cookie in a cookie jar, and reuse it for all subsequent requests.

### Mapping table

A SQLite database with tables:

```sql
CREATE TABLE user_map (rocket_id TEXT PRIMARY KEY, ruckchat_id TEXT, email TEXT, action TEXT);
CREATE TABLE room_map (rocket_id TEXT PRIMARY KEY, ruckchat_id TEXT, rocket_type TEXT, ruckchat_type TEXT);
CREATE TABLE message_map (rocket_id TEXT PRIMARY KEY, ruckchat_id TEXT);
CREATE TABLE file_map (rocket_id TEXT PRIMARY KEY, ruckchat_id TEXT, storage_path TEXT);
CREATE TABLE reaction_map (rocket_message_id TEXT, rocket_emoji TEXT, ruckchat_message_id TEXT, PRIMARY KEY (rocket_message_id, rocket_emoji));
CREATE TABLE checkpoints (stage TEXT PRIMARY KEY, last_id TEXT, completed_at TEXT);
```

### Pipeline stages

Execute in dependency order. Each stage is resumable from the checkpoint.

1. `source_inventory` — list users, rooms, teams, emoji, roles from RocketChat.
2. `users` — create or map to existing RuckChat users.
3. `organization_roles` — create custom roles and permissions.
4. `custom_emoji` — upload emoji images (or map to existing emoji).
5. `teams` — create teams and attach members.
6. `rooms` — create channels, groups, DMs, discussions. Archive where needed.
7. `room_memberships` — add members to channels/groups/teams.
8. `messages` — paginate room history, transform, and post in batches via `/admin/import`.
9. `threads` — resolve `tmid` to RuckChat `parent_id` using `message_map`.
10. `reactions` — migrate reactions using emoji mapping.
11. `files` — download from RocketChat and upload to RuckChat, then attach to messages.
12. `pins_and_stars` — migrate pinned/starred state if target API supports it.

### Dry-run logic

- Build the full `MigrationData` snapshot in memory.
- Compare against the mapping table to classify each entity as `create`, `update`, or `skip`.
- Do not call any mutating endpoint unless `--apply` is passed.
- Print a report:

```text
Dry-run report for RocketChat → RuckChat
==========================================
Users:            342 create, 18 update, 0 skip
Channels:         56 create, 2 update, 0 skip
Direct messages:  89 create, 0 update, 0 skip
Teams:            12 create, 0 update, 0 skip
Messages:         142,380 create, 1,204 update, 0 skip
Reactions:        8,921 create, 0 update, 0 skip
Files:            4,102 create, 0 update, 0 skip
Custom emoji:     24 create, 0 update, 0 skip
Roles:            7 create, 0 update, 0 skip

Run with --apply to write these changes.
```

### Interactive prompts

When config is incomplete or `--interactive` is set:

- Source URL and credentials
- Target URL and credentials
- Target organization selection
- Action for each inaccessible room: `skip`, `retry with elevation`, `abort`
- Confirmation before `--apply`

### Report

At the end of an applied run, write a JSON report:

```json
{
  "started_at": "2026-07-23T12:00:00Z",
  "completed_at": "2026-07-23T12:34:56Z",
  "source_url": "https://rocketchat.example.com",
  "target_url": "http://localhost:3000",
  "counts": {
    "users": { "created": 342, "updated": 18, "skipped": 0, "failed": 0 },
    "channels": { "created": 56, "updated": 2, "skipped": 0, "failed": 0 },
    "messages": { "created": 142380, "updated": 1204, "skipped": 0, "failed": 3 }
  },
  "mapping_store": "./rocketchat2ruckchat.mapping.sqlite",
  "failures": [
    { "stage": "files", "rocket_id": "abc", "error": "upload too large" }
  ]
}
```

## RocketChat → RuckChat mapping

| RocketChat concept | RuckChat target | Notes |
|--------------------|-----------------|-------|
| User               | `users`         | Map by email; preserve `username` as `display_name` prefix if unique; migrate `status` separately via presence API if desired. |
| Deleted user       | `users` with `deactivated_at` | Cannot log in, but messages retain authorship. |
| Role               | `organization_roles` + `permissions` | Built-in RocketChat roles (`admin`, `moderator`, `user`, `guest`, `bot`) map to RuckChat built-ins where possible; custom roles become `organization_roles`. |
| Permission         | `permissions` + `role_permissions` | Store key names; actual enforcement in RuckChat may lag until authz layer is made dynamic. |
| Public channel     | `channels` (is_private=false) | RocketChat `channels.*` endpoints. |
| Private group      | `channels` (is_private=true) | RocketChat `groups.*` endpoints. |
| Direct message     | `direct_message_conversations` | Resolve pairwise/triplet members. |
| Team               | `teams` + `team_rooms` + `team_memberships` | RocketChat `teams.*` endpoints. |
| Discussion         | `channels` (topic = parent name) | Discussions are modeled as regular channels. |
| Message            | `messages`      | Preserve original timestamp, author, edits, deletion, thread parent. |
| Thread             | `messages` with `parent_id` | Use `tmid` from RocketChat. |
| Reaction           | `reactions`     | Map emoji shortcodes through `custom_emoji` table. |
| File upload        | `files` + `message_files` | Download bytes, upload to RuckChat file endpoint, attach to message. |
| Custom emoji       | `custom_emoji`  | Upload image file and map shortcode. |
| Pinned message     | (deferred)      | RuckChat has no pin concept yet; report as skipped or store metadata. |
| Starred message    | (deferred)      | RuckChat has no star concept yet; report as skipped. |

## Rate limiting and backpressure

- Respect RocketChat `x-ratelimit-*` headers. Use exponential backoff on `429`.
- Respect RuckChat rate limits. The import endpoint may be expensive; for large instances, make it async with a status endpoint.
- Stream file downloads/uploads to avoid loading large files into memory.

## Security

- Never log RocketChat or RuckChat credentials.
- Use HTTPS for production source/target URLs; warn if plain HTTP is used.
- Store the mapping SQLite file with restrictive permissions (`0o600`).
- RuckChat import endpoint requires `Owner` or `Admin` role.

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Password hashes incompatible | Admin import endpoint accepts pre-hashed passwords only when the algorithm matches; otherwise users must reset passwords. |
| Large message history | Paginate RocketChat history and batch import calls. Consider async import endpoint for >100k messages. |
| File storage path mismatch | Download and re-upload file bytes rather than trusting `storage_path`. |
| Inaccessible rooms | Interactive prompt; default to skip with report. |
| Duplicate usernames/emails | Map to existing user by email; rename only if configured. |
| Schema drift | Keep the tool versioned and tied to `MigrationData` version. |

## Open questions remaining

1. Should the import endpoint be synchronous or asynchronous with a job status endpoint? (Recommendation: synchronous up to a configurable timeout/size, then queue.)
2. Should the standalone tool live inside the workspace or in a separate repository? (Recommendation: inside the workspace as `crates/rocketchat2ruckchat` for CI coordination, but publishable as its own binary.)
3. Does RuckChat want to adopt dynamic authorization based on `role_permissions`, or only store the matrix for migration parity? (Recommendation: store first; enforce later in a separate phase.)
4. How are RocketChat `@all`/`@here` mentions and custom user groups mapped? (Recommendation: convert to plain text mentions; skip user groups.)

## Next step

`/sc:implement` Phase A (server extensions) first, then Phase B (standalone tool).
