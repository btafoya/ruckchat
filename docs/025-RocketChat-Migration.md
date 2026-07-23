# 025-RocketChat-Migration

## Purpose

Provide a migration path from an existing RocketChat workspace into a RuckChat
organization. The tool is API-driven, idempotent, dry-run-first, and keeps all
RuckChat-side writes inside the existing service/repository layer.

## Requirements

- Read users, rooms (channels/groups/direct messages), messages, reactions,
  files, custom emoji, teams, roles, and permissions from the RocketChat REST API.
- Authenticate to RocketChat with a personal access token or username/password.
- Authenticate to RuckChat with email/password and reuse the session cookie.
- Produce a versioned `MigrationData` snapshot matching the server import format.
- Re-upload file and emoji bytes through the RuckChat file endpoint.
- Persist source→target ID mappings and resume checkpoints in SQLite.
- Default to dry-run; require `--apply` for writes.
- Generate a JSON report for both dry-run and applied runs.

## Design

The standalone `rocketchat2ruckchat` crate lives in `crates/rocketchat2ruckchat`.
It does not depend on `ruckchat-server` internals; it imports `ruckchat-domain`
and `ruckchat-id` for shared DTOs and defines a local `MigrationData` mirror.

Key modules:

- `config` — CLI (`--config`, `--apply`, `--dry-run`, `--interactive`,
  `--mapping-store`) and YAML configuration.
- `interactive` — prompts for missing source/target credentials and confirmation.
- `rocket_chat` — reqwest client, authentication, pagination, and response models.
- `ruckchat` — reqwest client with cookie jar, login, snapshot import, file
  upload, and file metadata fetch.
- `mapping` — SQLite mapping store for users, rooms, messages, files, emoji,
  roles, teams, and checkpoints.
- `transform` — in-memory conversion from RocketChat data to a RuckChat snapshot
  using deterministic UUIDv5 identifiers.
- `pipeline` — ordered inventory, transform, upload, checkpoint, and import
  stages.
- `report` — JSON report generation.

The RuckChat server side exposes `POST /api/v1/admin/organizations/:id/import`,
which writes the snapshot idempotently (`ON CONFLICT DO NOTHING`) and supports a
`dry_run` flag. This endpoint was implemented in Phase A.

## Acceptance Criteria

- `cargo check --workspace` and `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` pass.
- `cargo nextest run --workspace` passes.
- The binary compiles and can run in dry-run mode against a RocketChat source.
- The tool produces a deterministic mapping store and a JSON report.
- The tool respects `--apply` and writes to RuckChat only when it is supplied.
