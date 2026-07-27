# Design: RocketChat → RuckChat Migration Tool (v2 — Mongo-sourced, DB-to-DB)

## Status

This supersedes the original REST-based design. Its Phase A (admin import API,
`MigrationData` v2) and Phase B (`rocketchat2ruckchat` standalone tool) were
already implemented (`2747e15`, "Add admin import API and MigrationData v2 for
RocketChat migration (Phase A)") as a fully REST-based pipeline: RocketChat
REST client → transform → RuckChat admin REST import endpoint. This document
replaces that transport on **both** ends: source becomes a restored RocketChat
MongoDB dump read directly; target becomes a direct PostgreSQL write reusing
the existing transactional import logic. Nothing about `ruckchat-server`'s own
export/import CLI or its `POST /api/v1/admin/organizations/:id/import`
endpoint changes — they stay, unused by this tool, for their existing
backup/restore purpose.

## Scope

1. Migrate users, channels (public/private/DM), messages, reactions, files,
   and custom emoji from a **restored RocketChat mongodump** into a RuckChat
   organization by writing directly to RuckChat's PostgreSQL database.
2. Reuse `server/src/migrate.rs`'s transactional, idempotent
   `MigrationData`/`import()`/`validate()` logic rather than reimplementing it
   — extracted into a shared crate (see below) so both `ruckchat-server` and
   `rocketchat2ruckchat` depend on the same code.
3. Support re-runs: idempotent `ON CONFLICT DO NOTHING` writes (inherited from
   the shared import logic) plus the existing SQLite `rocket_id → ruckchat_id`
   mapping store for resumability.
4. Default to dry-run; writes require `--apply`.
5. Interactive prompts for the source dump path, target Postgres connection,
   target organization, and conflicts.

## Non-goals

- Live or replica-set Mongo connections — source is always a locally restored
  `mongorestore` target, never a running production `mongod`.
- Roles, permissions, and Teams migration. RocketChat's permission model is
  global (no per-organization scoping) and this and most instances carry only
  built-in roles; RuckChat's authorization is still hardcoded, so migrating
  the matrix has no enforcement value today. Teams migration is dropped for
  the same "can't validate against real data" reason — no instance surveyed
  has any Teams rooms. **The existing `organization_roles` / `permissions` /
  `role_permissions` / `teams` / `team_memberships` / `team_rooms` schema,
  `MigrationData` fields, and org-admin UI (`OrgAdminRoles.tsx`,
  `OrgAdminTeams.tsx`) are untouched** — they're independently-shipped
  org-admin features, not migration-only scaffolding, so `transform.rs`
  simply never populates those `MigrationData` arrays.
- System messages (RocketChat `t` values other than `null`: `au`, `uj`,
  `command`, `message_pinned`, `room_changed_privacy`, `r`,
  `discussion-created`, `livechat-close`/`livechat-started`,
  `livechat_navigation_history`). RuckChat has no system-message concept
  (`Message.author_id` is a required real user, no `is_system`/`message_type`
  discriminator exists), and these would import as plain, unstyled chat text
  indistinguishable from real conversation. Skipped entirely.
- Omnichannel/Livechat rooms (`rocketchat_room.t == "l"`), real-time sync,
  session/token/2FA migration.
- Non-GridFS file storage. RocketChat supports GridFS, AmazonS3, GoogleStorage,
  and FileSystem backends (`apps/meteor/server/lib/media/file-upload/config/`);
  this tool only reads GridFS-backed files (`store` field starting with
  `GridFS:`). A file whose `store` isn't GridFS is skipped and reported, not
  silently dropped.
- Real password migration. RocketChat's `services.password.bcrypt` hashes are
  bcrypt; RuckChat's `hash_password`/`verify_password`
  (`server/src/services/auth.rs`) are Argon2. A bcrypt string doesn't even
  parse as a valid Argon2 PHC hash, so copying it verbatim breaks login
  outright. No hash is carried over — see **Password handling** below.

## Why this changes

Switching the source from RocketChat's REST API to its Mongo dump, and the
target from RuckChat's REST admin-import endpoint to a direct Postgres write,
isn't a client swap — three things become possible that weren't before:

- **Byte-level file access.** RocketChat's REST API never exposes raw upload
  bytes in bulk; direct GridFS reads stream them straight out of the dump
  (`rocketchat_uploads.{files,chunks}`, `rocketchat_avatars.{files,chunks}`,
  etc. — confirmed present in a real dump).
- **No HTTP transport limits.** The current REST admin-import endpoint takes
  one JSON body per call; a 100k+ message workspace risks hitting body-size or
  timeout limits (flagged as an open risk in v1). Writing directly through
  `migrate::import`'s existing `sqlx::Transaction` has no such ceiling.
- **No running server required.** Neither RocketChat nor `ruckchat-server`
  needs to be a live process during migration — only a restored Mongo dump and
  a reachable Postgres connection string.

The tradeoff, accepted per the brainstorm discussion: schema-version coupling
to a specific RocketChat release (this design was validated against migration
version 335) and no service-layer authorization on the write side — but
`migrate::import` already bypasses that layer today for its existing
export/import use case, so this isn't new risk, just reused risk.

## Architecture

```text
┌───────────────────────────────────────────────────────────────────────┐
│                         rocketchat2ruckchat                           │
│        (standalone Rust binary, interactive or config-driven)         │
└───────────────┬─────────────────────────────────────┬─────────────────┘
                │ reads (mongodb driver)              │ writes (sqlx)
                ▼                                      ▼
┌──────────────────────────┐              ┌────────────────────────────┐
│  Restored RocketChat      │              │   RuckChat PostgreSQL       │
│  mongodump (local mongod) │              │   (direct connection)       │
└──────────────┬────────────┘              └──────────────┬─────────────┘
               │                                            │
               │  SQLite mapping table                      │
               │  rocket_id → ruckchat_id                   │
               │                                            │
               ▼                                            ▼
┌───────────────────────────────────────────────────────────────────────┐
│                       crates/ruckchat-migrate                         │
│   MigrationData, export()/import()/validate() — extracted from        │
│   server/src/migrate.rs, unchanged in logic, shared by both            │
│   ruckchat-server (CLI export/import, admin REST endpoint) and         │
│   rocketchat2ruckchat (this tool).                                     │
└───────────────────────────────────────────────────────────────────────┘
```

Also needed on the target side: the tool writes uploaded file bytes directly
to RuckChat's configured upload directory (`server/src/services/file.rs`
writes files as `<upload_dir>/<file_uuid>`, flat, no extension in the path —
confirmed: local-disk-only, no S3 support on the RuckChat side). The tool's
config must therefore include that same directory path so migrated files land
where `FileService` expects to find them, mirroring the `files.storage_path`
rows it inserts via `ruckchat-migrate`.

## Crate restructuring: extract `ruckchat-migrate`

Move `server/src/migrate.rs` (currently ~1500 lines: `MigrationData`,
`ImportCounts`, `export()`, `import()`, `validate()`, all `export_*`/`import_*`
helpers and row structs) into a new `crates/ruckchat-migrate` library crate,
depending only on `ruckchat-domain`, `ruckchat-id`, `ruckchat-common`, `sqlx`,
`serde`, `uuid`, `time`. `server/src/lib.rs` and `server/src/main.rs` switch
from `mod migrate;` to `ruckchat_migrate::{...}`; the admin import handler and
CLI `migrate export`/`migrate import` commands are otherwise unchanged.
`rocketchat2ruckchat` adds this crate as a dependency and calls
`ruckchat_migrate::import(&target_pool, &data, dry_run)` directly, building
its own `MigrationData` from Mongo instead of RocketChat REST JSON.

This is a mechanical move, not a rewrite of the import logic itself — the
transactional/idempotent behavior, `validate()`'s referential-integrity
checks, and `ON CONFLICT DO NOTHING` semantics carry over verbatim.

## Schema change: `parent_channel_id`

RocketChat discussions are rooms with a real parent-room link (`prid`), but
`channels` has no parent-channel concept. Add one:

```sql
-- migrations/migrations/<timestamp>_channel_parent.up.sql
ALTER TABLE channels ADD COLUMN parent_channel_id UUID REFERENCES channels(id) ON DELETE SET NULL;
```

```rust
// crates/ruckchat-domain/src/channel.rs
pub struct Channel {
    // ...existing fields...
    /// Parent channel, set when this channel originated as a RocketChat discussion.
    pub parent_channel_id: Option<ChannelId>,
}
```

`ruckchat-migrate`'s `ChannelRow`/`into_channel`/`import_channels` gain the
column. **No UI or authorization logic reads this field yet** — it's storage
only, so the relationship survives structurally without pretending to be
finished. TODO for a later phase: desktop/web UI to surface parent/child
channel navigation, and any authorization implications of nested channels.

## Password handling

RocketChat's bcrypt hash cannot become a working RuckChat Argon2 hash — a
bcrypt string doesn't even parse as a valid Argon2 PHC hash, so copying it
verbatim breaks login outright, not just weakens it. Instead, at
user-creation time `transform.rs` generates a real, usable, random temporary
password, hashes it the same way RuckChat's own signup flow does
(`hash_password`), and — per **Email support** below — emails it directly to
the user when `--send-emails` is passed. This supersedes an earlier draft of
this design that generated an unusable, never-communicated placeholder hash
relying on a post-migration admin action per user; that doesn't scale past a
handful of users, so credential delivery is now built into the migration tool
itself instead.

## Email support (Postmark)

Adding Postmark support (`postmark` crate v2.0.1, reqwest-backed, builder-style
client/request API) is now in scope, motivated directly by the password
problem above: RuckChat has **no email-sending capability of any kind**
today — confirmed by reading `server/src/services/server_admin.rs:236`
(`reset_password` generates a plaintext temporary password and returns it in
the JSON response with no delivery mechanism at all) and
`server/src/services/auth.rs`'s `register`/`login` (no verification or
forgot-password email flow exists either).

This becomes a new general-purpose crate, `crates/ruckchat-email`, not a
one-off inside the migration tool:

```rust
// crates/ruckchat-email/src/lib.rs (sketch)
pub struct EmailConfig {
    pub server_token: String,
    pub from_address: String,
}

pub struct EmailClient { /* wraps postmark::api::client::PostmarkClient */ }

impl EmailClient {
    pub fn new(config: &EmailConfig) -> Self { /* ... */ }

    /// Sends a migrated user their real temporary password.
    pub async fn send_migration_credentials(
        &self,
        to: &str,
        temp_password: &str,
    ) -> Result<(), EmailError> { /* inline HTML/text body, no Postmark template */ }

    /// Sends a server-admin-triggered password reset.
    pub async fn send_password_reset(
        &self,
        to: &str,
        temp_password: &str,
    ) -> Result<(), EmailError> { /* ... */ }
}
```

Bodies are composed inline in Rust (`api::Body::text`/`html`), not via
Postmark's server-side template feature — no Postmark-dashboard dependency,
copy changes go through a normal code deploy.

**`ruckchat-server` integration**: `ruckchat.yaml` gains an optional `email:`
section (`server_token`, `from_address`). When absent, email sending is a
no-op — `reset_password` keeps returning the plaintext password in its
response exactly as it does today (unchanged, graceful degradation, not a
hard error). When present, `server_admin.rs`'s `reset_password` additionally
calls `send_password_reset` after hashing; a failed send does **not** fail
the whole request (the password is still returned in the response as a
fallback) — it's recorded via the existing audit log alongside the
`user.password_reset` entry.

**`rocketchat2ruckchat` integration**: the tool's own config gains the same
`email:` section (its own copy, not read from the target's `ruckchat.yaml` —
the tool doesn't assume filesystem access to that file). A new `--send-emails`
flag (meaningful only alongside `--apply`) triggers
`send_migration_credentials` per migrated user immediately after the `users`
pipeline stage writes them. Without `--send-emails`, `--apply` still writes
everything and generates real usable passwords, but doesn't contact anyone —
letting an operator commit the data migration before deciding to notify
users, since these are two different kinds of consequence (a Postgres write
you can inspect/undo vs. an email you can't unsend).

Individual send failures don't abort the run: they're collected into the
migration report (`credential_emails: { sent, failed: [{email, error}] }`) so
the admin can follow up on just the handful that bounced, rather than the
whole migration failing over a few bad addresses.

## File storage

- **Source**: read bytes directly from Mongo GridFS buckets. Each upload
  document's `store` field (e.g. `"GridFS:Uploads"`) maps to a known bucket
  prefix — `rocketchat_uploads`, `rocketchat_avatars`, `rocketchat_userDataFiles`,
  `custom_sounds`, `assets` were all observed as real GridFS bucket pairs
  (`<prefix>.files`/`<prefix>.chunks`) in a live dump. Use the official
  `mongodb` driver crate's GridFS bucket API keyed by that prefix.
- **Target**: write bytes to the directory configured in RuckChat's own
  `ruckchat.yaml` (`uploads.directory` or equivalent — the tool's config must
  be told this same path), naming each file by its newly generated file UUID,
  matching `FileService::store`'s existing convention exactly. Then insert the
  `files` row (via `ruckchat-migrate`) with that `storage_path`.
- Files whose `store` isn't GridFS-prefixed are skipped and counted separately
  in the dry-run/apply report, not silently dropped.
- Custom emoji image storage bucket naming is unconfirmed — no custom emoji
  existed in the surveyed dump. Verify against a real custom-emoji-populated
  dump during implementation before assuming it shares the generic uploads
  bucket.

## Configuration (replaces the v1 YAML)

```yaml
source:
  mongo_uri: mongodb://localhost:27017
  database: rocketchat

target:
  database_url: postgres://ruckchat:ruckchat@localhost/ruckchat
  organization_id: 00000000-0000-0000-0000-000000000000
  upload_directory: /var/lib/ruckchat/uploads   # must match ruckchat.yaml

email:
  server_token: <postmark-server-token>
  from_address: no-reply@example.com

options:
  scope:
    - users
    - channels
    - messages
    - reactions
    - files
    - emoji
  map_existing_users: true
  deactivate_deleted_users: true
  archive_deleted_rooms: true
  dry_run: true

mapping_store: ./rocketchat2ruckchat.mapping.sqlite
```

All RocketChat/RuckChat REST auth sections (PAT, login, session cookie) are
removed — there is no HTTP client on either side anymore except the direct
Postmark API call for credential emails.

## Pipeline stages (revised)

Same ordered/resumable/checkpointed structure as v1, source and target swapped:

1. `source_inventory` — `count()` per Mongo collection for the dry-run report.
2. `users` — read `users`; map-or-create by email; `deactivated_at` set when
   `active: false`; generate a real usable temporary password and hash it
   (see **Password handling**). If `--send-emails`, email it via
   `ruckchat-email` immediately after the row is written.
3. `channels` — read `rocketchat_room`, filtering out `t: "l"` (livechat).
   `c`/`p` → channel; `d` → DM conversation; a room with `prid` set → channel
   with `parent_channel_id` pointing at the mapped parent.
4. `channel_memberships` — from `rocketchat_subscription`.
5. `messages` — per-room cursor pagination on `(rid, ts, _id)` (matches the
   existing `rid_1_ts_1__updatedAt_1` index); only `t: null` documents import,
   every other `t` value is skipped and counted.
6. `threads` — resolve `tmid` to the mapped parent `MessageId`.
7. `reactions` — extract each message's embedded `{":emoji:": {usernames}}`
   map and resolve each username against the user map.
8. `files` — GridFS stream to the target upload directory; non-GridFS `store`
   values are skipped and reported.
9. `custom_emoji` — as `files`, plus a `custom_emoji` row per shortcode.

Removed from v1: `organization_roles`, `custom_emoji`'s permission gating,
`teams`, `room_memberships`-for-teams, `pins_and_stars` (already deferred in
v1, stays deferred).

## Dependency changes (`crates/rocketchat2ruckchat/Cargo.toml`)

- Remove: `reqwest` (cookies/json/stream/multipart) — no more RocketChat or
  RuckChat HTTP clients.
- Add: `mongodb` (async, GridFS support), `bson`, `sqlx` (`postgres`,
  `runtime-tokio`, `macros` — matching `ruckchat-server`'s existing setup),
  `ruckchat-migrate = { path = "../ruckchat-migrate" }`,
  `ruckchat-email = { path = "../ruckchat-email" }`.
- Keep: `rusqlite` (mapping store), `dialoguer`/`console` (interactive
  prompts — target-organization selection becomes a direct `SELECT` against
  the target Postgres pool instead of a REST call), `clap`, `tokio`.
- `tokio-util` (`io-util`) was likely only needed for streaming REST download
  bodies — re-evaluate during implementation; GridFS's async stream may cover
  the same need without it.

## Risks and mitigations (revised)

| Risk | Mitigation |
|------|------------|
| Emailing hundreds of real users is irreversible | `--send-emails` is a separate, explicit flag from `--apply` — data can be migrated and inspected before anyone is contacted. |
| Individual credential emails fail (bad address, Postmark error) | Continue the run; collect failures in the migration report (`credential_emails.failed`) rather than aborting or silently dropping them. |
| Schema-version drift (validated against migration version 335) | Read the dump's `migrations` collection `version` field at startup; warn/abort on an unexpected version rather than silently misreading fields. |
| Custom emoji GridFS bucket naming unconfirmed | Verify against a real dump with custom emoji before shipping that stage; don't assume the generic uploads bucket. |
| Non-GridFS file storage instances | Explicit skip + report, not a silent gap — matches the "error clearly" decision from requirements discovery. |
| `parent_channel_id` has no UI/authorization consumer yet | Documented as storage-only with a follow-up TODO, not represented as a finished feature. |
| Postmark not yet configured on a given server | Email calls no-op gracefully; `reset_password` keeps returning the plaintext password as it does today, so nothing regresses for servers without Postmark set up. |

## Follow-ups explicitly out of scope for this design

- Updating `docs/ADR-012-Migration-and-Packaging.md` to describe the new
  `ruckchat-migrate` crate extraction, and likely a new ADR for
  `ruckchat-email`/Postmark as a new external dependency — do this alongside
  implementation, not as part of this design.
- Desktop/web UI and any authorization changes to surface `parent_channel_id`.
- Any future decision to migrate roles/permissions/Teams if a real instance
  using them becomes available for testing.
- Further email use cases beyond credential delivery and admin password
  resets (self-service forgot-password, signup verification, org-invite
  emails) — `ruckchat-email` is built general-purpose, but wiring it into
  those flows is separate follow-up work, not part of this migration task.

## Next step

`/sc:implement`: extract `ruckchat-migrate`, add the `parent_channel_id`
migration, then rewrite `rocketchat2ruckchat`'s source/target/transform/pipeline
modules per this design.
