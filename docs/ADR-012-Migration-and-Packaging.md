# ADR-012: Migration and Packaging Tools

## Status

Accepted — implemented in Phase 12.

## Context

RuckChat reached a point where teams need to move data between instances and
operators need repeatable deployment artifacts. Prior phases focused on runtime
features; Phase 11 (mobile Flutter client) was deferred, and Phase 12 was split
into concrete deliverables:

1. Define a versioned migration snapshot format for domain data.
2. Add a server CLI that exports and imports that snapshot idempotently.
3. Provide a production Docker image for the server.
4. Provide a script that builds cross-platform desktop installers and
   publishes releases on version tags.
5. Update packaging, migration, and operations documentation.

We needed to decide:

- How to represent users, organizations, channels, direct messages, messages,
  reactions, files, and message-file links in a portable format.
- How to import data without duplicating rows when run repeatedly.
- How to build the server without leaking `.env` secrets into the Docker image.
- How to serve the Web UI from the container.
- How to automate desktop releases for Linux (`.deb`/AppImage), macOS (`.dmg`),
  and Windows (`.msi`/NSIS).

## Decision

### Data migration snapshot

The server supports a versioned JSON migration snapshot with one top-level
object per aggregate:

- `version`: integer format version (currently `1`).
- `users`: array of user records with IDs, names, handles, emails, and
  Argon2 password hashes.
- `organizations`: array of organizations with IDs, names, and slugs.
- `memberships`: array of organization membership records (user, organization,
  role, created at).
- `settings`: array of organization settings records.
- `channels`: array of channels with topic and archived flag.
- `dm_conversations`: array of direct-message conversations with inline
  member IDs.
- `messages`: array of messages with channel/DM ID, sender, content, parent ID,
  timestamps, and conversation type.
- `reactions`: array of message reactions.
- `files`: array of file metadata records.
- `message_file_links`: array of links between messages and files.

The snapshot intentionally captures domain aggregates rather than raw database
rows so that downstream consumers can read and validate it without knowledge of
the internal schema.

### Idempotent import

Import runs inside a PostgreSQL transaction. Each row is inserted with
`ON CONFLICT DO NOTHING`, so re-importing the same snapshot is safe. The CLI
reports counts of rows inserted per aggregate and a dry-run mode previews the
counts without writing anything.

### Server container packaging

- The server is built as a multi-stage Docker image using
  `rust:1.94-bookworm` as the builder and `debian:bookworm-slim` as the runtime.
- SQLx offline mode (`SQLX_OFFLINE=true`) is used during the Docker build so the
  image compiles without a live database. The `.sqlx/query-*.json` metadata is
  checked into the repository and refreshed with `cargo sqlx prepare --workspace`.
- The runtime image includes `curl`, `ca-certificates`, and `libpq5`. A
  `HEALTHCHECK` probes `GET /` so orchestrators can detect a healthy container.
- A `docker-compose.yml` orchestrates PostgreSQL 17 and the server, mounting
  `ruckchat.yaml` plus named volumes for uploaded files and plugins.
- A single `scripts/publish.sh` script builds the Web UI assets, regenerates
  SQLx offline query data, and builds the Docker image (`--build-only` runs
  just this step).

### Desktop and release packaging

`scripts/publish.sh` is the sole release pipeline: it bumps versions,
generates a `CHANGELOG.md` entry, runs `cargo fmt`/`clippy`/`test`, builds the
server Docker image and a `cargo-deb` `.deb` package, builds Tauri desktop
bundles for the host platform (cross-compiling to any additional Rust targets
that happen to be installed locally), commits and GPG-signs the release,
pushes the tag, then publishes the Docker image to GHCR (`:VERSION` and
`:latest`) and creates the GitHub Release — uploading the server `.deb` and
every desktop bundle it built — via the `gh` CLI. `--dry-run`, `--build-only`,
`--publish-only`, `--no-build`, `--no-publish`, and `--no-desktop` gate which
phases run. This replaces the former split across
`scripts/release.sh`/`scripts/build-server.sh` and a
`.github/workflows/release.yml` CI job that rebuilt the Docker image and
created the release via `tauri-apps/tauri-action`; that workflow only ever
built Linux bundles on self-hosted runners (no genuine macOS/Windows CI
builds remained) and duplicated work `publish.sh` now does locally, so it was
removed rather than kept in parallel.

## Consequences

### Positive

- Operators can back up and restore domain data across RuckChat instances using
  a single JSON file.
- Docker deployment is first-class and reproducible, with no runtime dependency
  on a build-time database.
- Desktop releases are produced by a single script invocation, alongside the
  server Docker image, `.deb` package, and GitHub Release in one pipeline.
- SQLx offline metadata makes CI and Docker builds deterministic.

### Negative

- File payloads are not embedded in the migration snapshot; the snapshot only
  carries metadata. Restoring a full instance requires copying the file storage
  backend separately.
- `ON CONFLICT DO NOTHING` import means conflicting rows are silently skipped,
  not updated. The snapshot format is therefore import-once for existing IDs.
- The Docker image includes the Web UI embedded at the build revision; updating
  the UI requires rebuilding the image.
- macOS and Windows installers are unsigned until code-signing secrets are added
  to the repository. Users may see gatekeeper/smart-screen warnings.
- Cross-platform desktop bundles now depend on the operator's local machine
  having the relevant Rust cross-compilation targets installed; there is no
  longer a CI job that guarantees native macOS/Windows builds.

## Update: shared `ruckchat-migrate` crate

The snapshot types, export query, idempotent import, and dry-run accounting
originally in `server/src/migrate.rs` were extracted verbatim into
`crates/ruckchat-migrate` so the standalone `rocketchat2ruckchat` migration
tool (see `docs/DESIGN-RocketChat-Migration.md`) can call `import()` directly
against its own `PgPool`, writing straight to a target RuckChat Postgres
database without going through the REST admin-import endpoint or a running
`ruckchat-server` process. `ruckchat-server`'s own CLI (`migrate export`/
`migrate import`) and the `POST /api/v1/admin/organizations/:id/import`
endpoint are unchanged — both now depend on `ruckchat-migrate` instead of a
local module. `MigrationData`'s `Channel` entries also gained a
`parent_channel_id` field (nullable, storage-only) to preserve RocketChat
discussion → parent-room links during that migration.

## Implementation

- `crates/ruckchat-migrate/src/lib.rs` — snapshot types, export query,
  idempotent import, and dry-run accounting (moved from
  `server/src/migrate.rs`).
- `server/src/main.rs` — `clap` subcommands: `run` (default),
  `migrate export --output PATH`, and `migrate import --input PATH --dry-run`.
- `server/tests/migrate.rs` — integration tests for export shape, idempotent
  import, round-trip after truncate, and dry-run no-write.
- `Dockerfile` — multi-stage SQLx-offline build with `debian:bookworm-slim`
  runtime and health check.
- `.dockerignore` — excludes build artifacts, dependencies, and secrets while
  keeping `web/dist/` and `.sqlx/` available for the build context.
- `docker-compose.yml` — PostgreSQL 17 + server with health checks and volumes.
- `scripts/publish.sh` — builds web assets, refreshes `.sqlx/`, builds the
  Docker image and `.deb` package, builds desktop bundles, bumps versions,
  tags, and publishes to GHCR and GitHub Releases.
- `book/014-Deployment.md`, `book/015-Migration.md`, `book/016-Operations.md`,
  `server/README.md`, `desktop/README.md` — updated with packaging, migration,
  and release instructions.

## Related

- `book/014-Deployment.md`
- `book/015-Migration.md`
- `book/016-Operations.md`
- `server/README.md`
- `desktop/README.md`
- `docs/ADR-010-Runtime-YAML-Configuration.md`
- `docs/IMPLEMENTATION_PLAN.md`
