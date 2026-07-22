# RuckChat v1 Implementation Workflow

## 1. Source Documents

This workflow is derived from:

- `README.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `book/*.md` (production handbook)
- `docs/requirements/RUCKCHAT-REQUIREMENTS.md`
- `docs/design/ARCHITECTURE-DESIGN.md`
- `docs/design/DATABASE-SCHEMA-DESIGN.md`
- `docs/design/OPENAPI-DESIGN.md`

## 2. Workflow Strategy

- **Approach:** Systematic, incremental implementation following the implementation contract in `CLAUDE.md`.
- **Order:** Cargo workspace → shared crate → database/migrations → domain models → services → REST API → WebSocket → desktop → mobile → search → files → notifications → plugin SDK → packaging.
- **Quality gates:** Every phase ends with tests, documentation, OpenAPI updates, and passing `fmt`/`clippy`.

## 3. Phase Overview

| Phase | Focus | Main Deliverables | Validation |
|-------|-------|-------------------|------------|
| 0 | Preparation | Toolchain, repo setup, CI skeleton | Local build works |
| 1 | Workspace bootstrap | Cargo workspace, shared crate, config | `cargo build` passes |
| 2 | Database and migrations | `migrations/` crate, first schema | `cargo sqlx migrate run` works |
| 3 | Domain models | DTOs, error types, validation in `shared` | Unit tests pass |
| 4 | Server foundation | Axum app, state, middleware, health endpoints | `cargo test` passes |
| 5 | Authentication | Register, login, logout, password reset | Integration tests pass |
| 6 | Organizations | CRUD, invitations, roles, membership | Integration tests pass |
| 7 | Channels | Public/private channels, membership | Integration tests pass |
| 8 | Direct messages | DM creation, membership | Integration tests pass |
| 9 | Messaging | Send, edit, delete, threads, reactions | Integration + WS tests pass |
| 10 | WebSocket | Connection manager, event routing | WS integration tests pass |
| 11 | Search | PostgreSQL full-text search endpoint | Search tests pass |
| 12 | File uploads | Local storage, upload/download, quotas | File tests pass |
| 13 | Notifications | Unread counters, email queue, desktop events | Notification tests pass |
| 14 | Plugin SDK | Plugin loader, host API, sample plugin | Plugin tests pass |
| 15 | MCP server | SSE endpoint, tools, auth | MCP tests pass |
| 16 | Desktop client | Tauri + React scaffold, auth, channels, messages | Vitest + manual smoke |
| 17 | Mobile client | Flutter scaffold, auth, channels, messages | Flutter tests + smoke |
| 18 | Packaging | Docker image, desktop installers, CI pipeline | CI green |
| 19 | Release readiness | Final tests, docs, release checklist | All gates pass |

## 4. Detailed Phase Plans

### Phase 0: Preparation

**Goal:** Establish the development environment and tooling.

**Tasks:**

1. Verify Rust toolchain (stable) and `cargo`.
2. Install `sqlx-cli` for migrations.
3. Install PostgreSQL 15+ locally or via Docker Compose.
4. Install Node.js tooling for desktop client (`pnpm`, `vite`).
5. Install Flutter SDK for mobile client.
6. Add `.gitignore` for Rust, Node, Flutter, and environment files.
7. Create `docker-compose.yml` for PostgreSQL and optional object storage (MinIO for tests).
8. Create initial CI workflow skeleton (GitHub Actions) with `fmt`, `clippy`, and `test` jobs.

**Validation:**

- `docker compose up db` starts PostgreSQL.
- `cargo --version` and `sqlx --version` work.
- CI skeleton is present and syntactically valid.

### Phase 1: Workspace Bootstrap

**Goal:** Create the Cargo workspace and `shared` crate.

**Tasks:**

1. Update root `Cargo.toml` with workspace metadata and resolver.
2. Create `shared/Cargo.toml` with dependencies:
   - `serde`, `serde_json`
   - `uuid` with `serde` and `v4`
   - `chrono` with `serde`
   - `validator`
   - `thiserror`
3. Create `server/Cargo.toml` with dependencies:
   - `axum`, `tokio`, `tower`, `tower-http`
   - `sqlx` with `runtime-tokio-rustls`, `postgres`, `migrate`, `uuid`, `chrono`
   - `argon2`, `password-hash`
   - `tracing`, `tracing-subscriber`
   - `figment` or `envy`
   - `jsonwebtoken` or cookie signing crate
   - `tokio-tungstenite` or `axum` built-in WebSocket support
   - `reqwest` or `lettre` for SMTP
   - `aws-sdk-s3` alternative for S3 (optional, use `rust-s3` or similar)
4. Add `migrations/` crate or use `sqlx` migration files directly.
5. Configure `rustfmt.toml` and `.clippy.toml` if needed.

**Validation:**

- `cargo build --workspace` compiles.
- `cargo fmt --check` passes.
- `cargo clippy --workspace -- -D warnings` passes (will be empty after stubs removed).

### Phase 2: Database and Migrations

**Goal:** Establish the PostgreSQL schema and migration tooling.

**Tasks:**

1. Create `migrations/YYYYMMDDHHMMSS_create_users.up.sql` and `.down.sql`.
2. Create remaining migrations in dependency order:
   - `users`
   - `organizations`
   - `organization_memberships`
   - `sessions`
   - `user_preferences`
   - `organization_settings`
   - `channels`
   - `channel_memberships`
   - `direct_message_conversations`
   - `dm_members`
   - `messages`
   - `reactions`
   - `files`
   - `message_attachments`
   - `email_notification_queue`
3. Add GIN index and `content_tsv` generation for messages.
4. Add `sqlx-data.json` preparation step.

**Validation:**

- `cargo sqlx migrate run` applies cleanly.
- `cargo sqlx migrate revert` reverts one step and re-applies cleanly.
- `cargo sqlx prepare` generates metadata successfully.

### Phase 3: Domain Models

**Goal:** Define shared DTOs, domain types, errors, and validation.

**Tasks:**

1. Define request/response DTOs for all MVP resources.
2. Define WebSocket event types and envelope.
3. Define application error codes and error response body.
4. Add validation constants and helpers.
5. Add unit tests for validators.

**Validation:**

- `cargo test -p shared` passes.
- All DTOs derive `Serialize`, `Deserialize`, and `Validate` where applicable.

### Phase 4: Server Foundation

**Goal:** Build the Axum application skeleton.

**Tasks:**

1. Create `server/src/main.rs` with configuration loading and tracing setup.
2. Create `server/src/config.rs` with environment mapping.
3. Create `server/src/state.rs` with `AppState` holding `PgPool`, config, etc.
4. Create `server/src/error.rs` with `AppError` and HTTP mapping.
5. Create `server/src/router.rs` mounting routes and middleware.
6. Implement `/health` and `/health/ready`.
7. Add request logging middleware.

**Validation:**

- Server starts and responds to `/health`.
- `cargo test -p server` passes for health endpoints.

### Phase 5: Authentication

**Goal:** Implement user registration, login, logout, and password reset.

**Tasks:**

1. Implement `Argon2id` password hashing.
2. Implement session token generation and cookie handling.
3. Create `users` repository.
4. Create `auth_service` with register, login, logout.
5. Implement password reset token generation and email stub.
6. Implement handlers:
   - `POST /auth/register`
   - `POST /auth/login`
   - `POST /auth/logout`
   - `POST /auth/password-reset/request`
   - `POST /auth/password-reset/confirm`
   - `GET /auth/me`
7. Add auth middleware extracting user from session.
8. Add rate-limit middleware for auth endpoints.
9. Write integration tests for all auth flows.

**Validation:**

- Integration tests for register, login, logout, and password reset pass.
- Rate limiting behavior verified.
- `cargo clippy` clean.

### Phase 6: Organizations

**Goal:** Implement multi-tenant organizations and RBAC.

**Tasks:**

1. Create `organizations` repository.
2. Create `organization_service` with CRUD, invitations, membership.
3. Implement role checks and invariant enforcement (at least one owner).
4. Implement handlers:
   - `POST /organizations`
   - `GET /organizations`
   - `GET /organizations/{id}`
   - `PATCH /organizations/{id}`
   - `DELETE /organizations/{id}`
   - `GET /organizations/{id}/members`
   - `POST /organizations/{id}/invitations`
   - `POST /organizations/{id}/invitations/{token}/accept`
   - `DELETE /organizations/{id}/members/{user_id}`
   - `PATCH /organizations/{id}/members/{user_id}/role`
5. Add integration tests.
6. Update OpenAPI spec.

**Validation:**

- All organization endpoints tested.
- RBAC tests confirm owner/admin/member differences.

### Phase 7: Channels

**Goal:** Implement public and private channels.

**Tasks:**

1. Create `channels` repository.
2. Create `channel_service` with CRUD, membership, join/leave.
3. Enforce channel name rules and uniqueness.
4. Implement handlers:
   - `GET /organizations/{id}/channels`
   - `POST /organizations/{id}/channels`
   - `GET /channels/{id}`
   - `PATCH /channels/{id}`
   - `DELETE /channels/{id}`
   - `POST /channels/{id}/join`
   - `POST /channels/{id}/leave`
   - `GET /channels/{id}/members`
5. Add integration tests.
6. Update OpenAPI spec.

**Validation:**

- Public/private access rules tested.
- Join/leave behavior tested.
- Unique channel name per organization tested.

### Phase 8: Direct Messages

**Goal:** Implement one-to-one and group DMs.

**Tasks:**

1. Create `direct_message_conversations` repository.
2. Create `dm_service` with creation and membership rules.
3. Implement handlers:
   - `GET /organizations/{id}/dms`
   - `POST /organizations/{id}/dms`
   - `GET /dms/{id}`
4. Add integration tests.
5. Update OpenAPI spec.

**Validation:**

- DM creation tested.
- Duplicate DM prevention tested if implemented.
- Membership immutability tested.

### Phase 9: Messaging

**Goal:** Implement message creation, editing, deletion, threads, and reactions.

**Tasks:**

1. Create `messages` repository.
2. Create `message_service` with send, edit, delete, thread replies, reactions.
3. Implement mention parsing and notification queueing.
4. Implement handlers:
   - `GET /conversations/{id}/messages`
   - `POST /conversations/{id}/messages`
   - `GET /messages/{id}`
   - `PATCH /messages/{id}`
   - `DELETE /messages/{id}`
   - `GET /messages/{id}/replies`
   - `POST /messages/{id}/reactions`
   - `DELETE /messages/{id}/reactions/{emoji}`
5. Add integration tests for each operation.
6. Update OpenAPI spec.

**Validation:**

- Send, edit, delete flows tested.
- Thread reply flow tested.
- Reaction add/remove and uniqueness tested.
- Mention notification queueing tested.

### Phase 10: WebSocket

**Goal:** Implement real-time event delivery.

**Tasks:**

1. Create `websocket/manager.rs` with connection registry.
2. Create `websocket/connection.rs` for per-connection handling.
3. Create `websocket/events.rs` with event types.
4. Integrate WebSocket route into Axum router.
5. Emit events from services:
   - `message.created`
   - `message.updated`
   - `message.deleted`
   - `reaction.updated`
   - `typing.updated`
   - `presence.updated`
6. Implement ping/pong and graceful shutdown event.
7. Add integration tests with a test WebSocket client.

**Validation:**

- WS connection authenticated and receives events.
- Typing debouncing tested.
- Reconnect behavior tested.

### Phase 11: Search

**Goal:** Implement PostgreSQL full-text search.

**Tasks:**

1. Verify `content_tsv` column and GIN index.
2. Create `search_service` with query building and ranking.
3. Implement handler:
   - `GET /organizations/{id}/search`
4. Enforce organization and conversation scoping.
5. Add integration tests.
6. Update OpenAPI spec.

**Validation:**

- Search returns relevant results.
- Results respect membership permissions.
- Pagination works.

### Phase 12: File Uploads

**Goal:** Implement file uploads and storage.

**Tasks:**

1. Create `file_storage` abstraction with local and S3 backends.
2. Implement local filesystem backend first.
3. Create `files` repository and `file_service`.
4. Implement handlers:
   - `POST /files`
   - `GET /files/{id}`
   - `GET /files/{id}/download`
   - `DELETE /files/{id}`
5. Add MIME type and size validation.
6. Add image thumbnail generation (optional, post-MVP if risky).
7. Add integration tests.
8. Update OpenAPI spec.

**Validation:**

- Upload, download, delete tested.
- Quota enforcement tested.
- Unauthenticated access denied.

### Phase 13: Notifications

**Goal:** Implement unread counters, email notifications, and desktop events.

**Tasks:**

1. Create `email_notification_queue` repository.
2. Create `notification_service`.
3. Queue mentions and DMs for email when recipient offline.
4. Implement background email task using SMTP config.
5. Add unread counters on channels and DMs.
6. Wire desktop notifications through WebSocket events.
7. Add integration tests.
8. Update OpenAPI spec.

**Validation:**

- Unread counters update on new messages.
- Email queue populated for offline mentions.
- Background task sends emails.

### Phase 14: Plugin SDK

**Goal:** Implement the native plugin system.

**Tasks:**

1. Create `plugins` crate with SDK trait and types.
2. Define plugin ABI and entry point macro.
3. Create `plugin_host` in server with load/init/run/shutdown lifecycle.
4. Implement event hooks and command registration.
5. Create a sample plugin in `examples/`.
6. Add tests for plugin load and hook invocation.
7. Update documentation.

**Validation:**

- Server loads sample plugin.
- Plugin receives `on_message_received` events.
- Plugin failure does not crash server.

### Phase 15: MCP Server

**Goal:** Expose MCP over SSE.

**Tasks:**

1. Add MCP route `/mcp/v1/sse`.
2. Implement MCP session and tool dispatch.
3. Add tools: `list_channels`, `list_direct_messages`, `get_messages`, `search_messages`, `post_message`.
4. Enforce same authorization as REST API.
5. Add tests.
6. Update documentation.

**Validation:**

- MCP endpoint authenticates and returns events.
- Tools respect permissions.
- `post_message` requires confirmation by default.

### Phase 16: Desktop Client

**Goal:** Build Tauri + React client.

**Tasks:**

1. Initialize Tauri project in `desktop/`.
2. Configure React, TypeScript, Tailwind CSS, Vite.
3. Implement auth screens (login/register).
4. Implement organization/channel/DM list.
5. Implement message view and composer.
6. Connect to WebSocket.
7. Add native notifications.
8. Add unit tests with Vitest and React Testing Library.
9. Add manual smoke checklist.

**Validation:**

- `pnpm test` passes.
- Client logs in and receives messages.
- Desktop notifications fire on mentions.

### Phase 17: Mobile Client

**Goal:** Build Flutter client.

**Tasks:**

1. Initialize Flutter project in `mobile/`.
2. Configure Riverpod for state management.
3. Implement auth screens.
4. Implement conversation list and chat view.
5. Connect to WebSocket.
6. Add foreground notifications.
7. Add widget tests.
8. Add manual smoke checklist.

**Validation:**

- `flutter test` passes.
- App logs in and receives messages.
- Reconnect fetches missed history.

### Phase 18: Packaging

**Goal:** Create distributable artifacts.

**Tasks:**

1. Write `Dockerfile` for server.
2. Write `docker-compose.yml` for full stack.
3. Add GitHub Actions jobs for:
   - Server test and clippy
   - Desktop build (Linux, macOS, Windows)
   - Mobile build (Android, iOS)
   - Docker image publish
4. Add desktop release scripts.
5. Add mobile signing steps (CI secrets).
6. Update deployment documentation.

**Validation:**

- Docker image builds and runs.
- CI pipeline is green.
- Desktop installer produced.

### Phase 19: Release Readiness

**Goal:** Ensure the product is ready for v1.0.0.

**Tasks:**

1. Run full test suite.
2. Verify all handbook chapters are current.
3. Verify OpenAPI spec matches implementation.
4. Verify no `TODO` markers in source.
5. Run security checklist (secrets scan, dependency audit).
6. Complete release checklist.
7. Tag release.

**Validation:**

- All tests pass.
- CI green.
- Documentation complete.
- Release tagged.

## 5. Dependency Graph

```
Phase 0 ──▶ Phase 1 ──▶ Phase 2 ──▶ Phase 3 ──▶ Phase 4
                                                 │
            Phase 5 ◀─────────────────────────────┤
              │
            Phase 6 ──▶ Phase 7 ──▶ Phase 8 ──▶ Phase 9 ──▶ Phase 10
                                                            │
            Phase 11 ◀──────────────────────────────────────┤
            Phase 12 ◀──────────────────────────────────────┤
            Phase 13 ◀──────────────────────────────────────┤
            Phase 14 (parallel after Phase 10)
            Phase 15 (parallel after Phase 9)
            Phase 16 (parallel after Phase 10)
            Phase 17 (parallel after Phase 10)
            Phase 18 (depends on server and client builds)
            Phase 19 (depends on all prior phases)
```

## 6. Quality Gates

Every phase must pass:

1. `cargo fmt --check` (server/shared)
2. `cargo clippy --workspace -- -D warnings`
3. Unit tests for new code
4. Integration tests for new endpoints/behaviors
5. Updated OpenAPI spec for API changes
6. Updated handbook if behavior changes
7. No `TODO` markers in new code

Client phases additionally require:

1. `pnpm lint && pnpm test` (desktop)
2. `flutter test` (mobile)

## 7. Risk Management

| Risk | Mitigation |
|------|------------|
| SQLx compile-time checks slow iteration | Prepare query metadata in CI; use `SQLX_OFFLINE` for builds |
| WebSocket state limits scaling | Accepted v1 constraint; monitor connection counts |
| Plugin SDK ABI stability | Version plugin API; bump major on breaking changes |
| Flutter/Tauri toolchain complexity | Pin versions in CI and README |
| Email deliverability | Use configurable SMTP; queue and retry failures |

## 8. Files Produced

- `claude/workflows/RUCKCHAT_V1_WORKFLOW.md` (this file)

## 9. Next Step

After approval, execute the workflow with `/sc:implement`. The recommended first implementation task is **Phase 1: Workspace Bootstrap**.
