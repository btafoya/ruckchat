# RuckChat Test Plan

This document describes how to verify that a RuckChat build is ready for use.
It covers automated test execution, server startup checks, and manual smoke tests
for the server, desktop client, and Web UI.

## 1. Scope

RuckChat is a self-hosted team chat platform. The runtime stack is:

- `ruckchat-server` — Rust Axum server with PostgreSQL, WebSocket, MCP, plugins,
  Web Push, and static Web UI serving.
- `ruckchat-desktop` — Tauri v2 + React client.
- `ruckchat-web` — Vite React Progressive Web App.

This test plan applies to:

- Local development builds before a commit.
- Pre-release validation via `scripts/release.sh`.
- Production deployment smoke tests (Docker Compose).

## 2. Test Environments

### 2.1 Required services

| Service | Version | Purpose |
|---------|---------|---------|
| Rust toolchain | 1.94+ | Workspace build and tests |
| PostgreSQL | 17+ | Server data store |
| Node.js / pnpm | Latest LTS / 9+ | Desktop and Web UI builds |

### 2.2 Databases

- `DATABASE_URL` — used at compile time by SQLx macros and at runtime by server
  integration tests.
- `RUCKCHAT_TEST_ADMIN_DATABASE_URL` — used by `migrations/tests/schema.rs` to
  create isolated per-test databases.

Use the provided `.env.testing` file when testing locally:

```bash
set -a
source .env.testing
set +a
```

Default values from `.env.testing`:

```text
DATABASE_URL=postgres://ruckchat:ruckchat@localhost:5445/ruckchat
RUCKCHAT_TEST_ADMIN_DATABASE_URL=postgres://postgres:postgres@localhost:5445/postgres
```

### 2.3 Test artifacts

- `target/` — Rust build output.
- `desktop/node_modules/` and `desktop/src-tauri/target/` — desktop build output.
- `web/node_modules/` and `web/dist/` — Web UI build output.
- `.sqlx/` — SQLx offline query cache used by Docker builds.

## 3. Automated Test Execution

Run the automated suites in this order. Stop at the first failure.

### 3.1 Workspace formatting and linting

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected result: no errors, no warnings.

### 3.2 Workspace unit tests

```bash
cargo nextest run --workspace
```

If `cargo nextest` is not installed:

```bash
cargo test --workspace
```

Expected result: all tests pass.

### 3.3 Server integration tests

Requires a running PostgreSQL database and `DATABASE_URL`.

```bash
set -a
source .env.testing
set +a
cargo test -p ruckchat-server
```

Expected result: all integration tests in `server/tests/` pass.

### 3.4 Migration/schema tests

```bash
set -a
source .env.testing
set +a
cargo test -p ruckchat-migrations
```

Expected result: migrations apply cleanly to isolated per-test databases.

### 3.5 Desktop client type check and unit tests

```bash
cd desktop
pnpm install
pnpm typecheck
pnpm test
```

Expected result: no TypeScript errors and all Vitest tests pass.

### 3.6 Web UI type check and unit tests

```bash
cd web
pnpm install
pnpm typecheck
pnpm test
```

Expected result: no TypeScript errors and all Vitest tests pass.

### 3.7 End-to-end automated checks

After the above suites pass, build the production server binary with embedded
Web UI assets:

```bash
cd web
pnpm build
cd ..
cargo build --release -p ruckchat-server
```

Expected result: the release binary is produced at
`target/release/ruckchat-server`.

## 4. Server Startup and Smoke Tests

### 4.1 Generate and review configuration

```bash
cargo run -p ruckchat-server -- --init-config ./ruckchat.yaml
```

Expected result: a default `ruckchat.yaml` file is written. Review the file and
confirm at minimum:

- `database.url` points to a reachable PostgreSQL instance.
- `base_url` matches the URL clients will use (default: `http://localhost:3000`).
- `files.path` is a writable directory.
- `plugins.path` is empty or points to a directory of trusted compiled plugins.

### 4.2 Start the server

```bash
cargo run -p ruckchat-server -- --config ./ruckchat.yaml
```

Expected result: the server logs migration success, binds to the configured
address, and is reachable at the `base_url`.

### 4.3 Health check

```bash
curl -s http://localhost:3000/ | head -n 20
```

Expected result: the Web UI `index.html` is served or the server responds
without error. If `web.path` is not configured and the Web UI was not built,
verify that API-only requests still succeed.

### 4.4 API smoke tests

Use `curl`, the OpenAPI spec, or any HTTP client. Examples use `localhost:3000`.

#### 4.4.1 Register a new user and organization

```bash
curl -s -X POST http://localhost:3000/auth/register \
  -H 'content-type: application/json' \
  -d '{"email":"test-1@example.com","password":"Correct-Horse-Battery-Staple!99","name":"Test User"}'
```

Expected result: `201 Created` with JSON containing `user`, `organization`,
`token`, and a `Set-Cookie: ruckchat_session=...` header.

Save the `token` value for subsequent authenticated requests.

#### 4.4.2 Get the authenticated profile

```bash
curl -s -H 'authorization: Bearer <token>' http://localhost:3000/users/me
```

Expected result: `200 OK` with the registered user profile.

#### 4.4.3 List organizations

```bash
curl -s -H 'authorization: Bearer <token>' http://localhost:3000/organizations
```

Expected result: `200 OK` with an array containing the initial organization.

#### 4.4.4 Create a channel

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"name":"general","topic":"Team-wide discussion"}' \
  http://localhost:3000/organizations/<organization_id>/channels
```

Expected result: `201 Created` with channel details.

#### 4.4.5 Post a message

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"content":"Hello, RuckChat!"}' \
  http://localhost:3000/channels/<channel_id>/messages
```

Expected result: `201 Created` with the created message.

#### 4.4.6 Fetch message history

```bash
curl -s -H 'authorization: Bearer <token>' \
  'http://localhost:3000/channels/<channel_id>/messages?limit=20'
```

Expected result: `200 OK` with a `messages` array that includes the message
posted above.

#### 4.4.7 Add a reaction

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"emoji":"🚀"}' \
  http://localhost:3000/messages/<message_id>/reactions
```

Expected result: `201 Created` or `200 OK` depending on implementation.

#### 4.4.8 Start a direct message conversation

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"user_id":"<other_user_id>"}' \
  http://localhost:3000/direct-messages
```

Expected result: `201 Created` with a direct-message conversation.

#### 4.4.9 Logout

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  http://localhost:3000/auth/logout
```

Expected result: `204 No Content`. A follow-up request with the same token must
return `401 Unauthorized`.

### 4.5 WebSocket smoke test

Open a WebSocket connection to `ws://localhost:3000/websocket` with the
`Authorization: Bearer <token>` header or after establishing the
`ruckchat_session` cookie. Then:

1. Wait for a `connection.established` event.
2. Send a typing indicator:
   `{"type":"typing","conversation_id":"<channel_id>","conversation_type":"channel"}`.
3. Have a second authenticated client post a message in the same channel.
4. Verify the first client receives `message.created` with the new message.

Expected result: real-time events arrive as JSON envelopes with `type`, `id`,
`timestamp`, and `payload` fields.

### 4.6 File upload smoke test

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  -F 'file=@/path/to/test-file.png' \
  http://localhost:3000/files
```

Expected result: `201 Created` with file metadata including `id`, `filename`,
and `url`. The file content must be retrievable at the returned URL.

### 4.7 Plugin slash command smoke test

If a plugin is loaded and exposes a command named `hello` under plugin name
`demo`:

```bash
curl -s -X POST \
  -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"channel_id":"<channel_id>","args":{}}' \
  http://localhost:3000/plugins/demo/commands/hello
```

Expected result: `200 OK` with the plugin response text.

### 4.8 MCP endpoint smoke test

```bash
curl -s -N \
  -H 'authorization: Bearer <token>' \
  -H 'accept: text/event-stream' \
  http://localhost:3000/mcp/v1/sse
```

Expected result: a stream of SSE events beginning with an `endpoint` event.

## 5. Desktop Client Smoke Tests

### 5.1 Development startup

```bash
cd desktop
pnpm install
pnpm tauri dev
```

Expected result: the Vite dev server starts on `http://localhost:5173`, the
Tauri WebView opens, and the client connects to `http://localhost:3000` by
default.

### 5.2 Authentication flow

1. Start the server with a fresh database or an existing test user.
2. In the desktop login screen, enter the test user email and password.
3. Click sign in.

Expected result: the main three-pane chat interface appears, the sidebar lists
the user's organizations, and the server logs a new WebSocket connection.

### 5.3 Channel and message flow

1. Select an organization.
2. Select a channel from the sidebar.
3. Type a message in the composer and press Enter.

Expected result: the message appears in the message pane, paginated history
loads on scroll, and a typing indicator is visible to other connected clients.

### 5.4 Thread replies

1. Hover over an existing message and select the reply action.
2. In the thread pane, type a reply and send.

Expected result: the reply appears under the parent message in the thread pane
and is reflected in the channel history.

### 5.5 Reactions

1. Hover over a message and add a `🚀` reaction.
2. Verify the reaction count appears next to the message.

Expected result: the reaction is persisted and visible to other clients in real
time.

### 5.6 File attachment

1. In the composer, click the attachment button and select a file through the
   native dialog.
2. Send the message.

Expected result: the file metadata is attached to the message and the file is
uploaded to the server.

### 5.7 Settings and backend URL

1. Navigate to `/settings`.
2. Change the backend URL to a different reachable server.
3. Save.

Expected result: subsequent API calls and WebSocket connections use the new URL.
An empty value falls back to `http://localhost:3000`.

### 5.8 Notifications and tray

1. Enable notifications in `/settings`.
2. Have another user mention the test user in a channel.

Expected result: an OS-native notification appears, and the tray icon tooltip
reflects an updated unread count.

### 5.9 Deep links

1. While the app is running, open a `ruckchat://` URL from the OS.

Expected result: the app handles the URL and navigates to the referenced
resource.

### 5.10 Offline resilience

1. Disconnect the network or stop the server.
2. Type a message in the composer but do not send.
3. Restore the network or restart the server.
4. Send the message.

Expected result: the draft is preserved, the message sends on retry, and failed
messages show a retry affordance.

## 6. Web UI Smoke Tests

### 6.1 Development startup

```bash
cd web
pnpm install
pnpm dev
```

Expected result: the Vite dev server starts and proxies API requests to
`http://localhost:3000`.

### 6.2 PWA install prompt

1. Open the Web UI in a supported browser.
2. Confirm the install prompt or add-to-home-screen option appears.

Expected result: the PWA manifest and service worker are registered.

### 6.3 Web Push notifications

1. Allow browser notification permission.
2. Subscribe to push notifications.
3. Have another user send a DM or mention the test user.

Expected result: a browser push notification is received.

### 6.4 Responsive layout

1. Resize the browser to a narrow viewport.

Expected result: the hamburger menu opens the sidebar overlay and the composer
remains usable.

### 6.5 Shared component behavior

Repeat the channel, message, thread, reaction, and settings tests from
Section 5 in the browser.

Expected result: behavior matches the desktop client except where native Tauri
capabilities (tray, OS notifications, native file dialog) are replaced by web
APIs.

## 7. Docker Compose Smoke Tests

### 7.1 Build the server image

```bash
./scripts/build-server.sh
```

Expected result: the image `ruckchat-server:latest` is built with embedded Web
UI assets and SQLx offline metadata.

### 7.2 Run with Docker Compose

```bash
# Create ruckchat.yaml first if it does not exist.
cargo run -p ruckchat-server -- --init-config ./ruckchat.yaml
# Edit ./ruckchat.yaml to use the compose service names if needed.
docker compose up -d
```

Expected result: PostgreSQL becomes healthy, the server container starts, and
`http://localhost:3000` serves the Web UI and API.

### 7.3 Compose build variant

```bash
docker compose -f docker-compose.build.yml up -d
```

Expected result: the server image is built from source and the stack starts
successfully.

### 7.4 Tear down

```bash
docker compose down -v
```

Expected result: containers and named volumes are removed without errors.

## 8. Domain Data Migration Tests

### 8.1 Export a snapshot

```bash
cargo run -p ruckchat-server -- --config ./ruckchat.yaml migrate export --output export.json
```

Expected result: a JSON snapshot file is written with domain data and a version
field.

### 8.2 Import a snapshot

```bash
cargo run -p ruckchat-server -- --config ./ruckchat.yaml migrate import --input export.json
```

Expected result: data imports idempotently. Re-running the import must not
duplicate rows.

### 8.3 Dry-run import

```bash
cargo run -p ruckchat-server -- --config ./ruckchat.yaml migrate import --input export.json --dry-run
```

Expected result: the command reports what would be imported without writing to
the database.

## 9. Release Validation Checklist

Run before tagging a release:

- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo nextest run --workspace` passes.
- [ ] `cargo test -p ruckchat-server` passes with `DATABASE_URL` set.
- [ ] `cargo test -p ruckchat-migrations` passes with
  `RUCKCHAT_TEST_ADMIN_DATABASE_URL` set.
- [ ] `cd desktop && pnpm typecheck && pnpm test` passes.
- [ ] `cd web && pnpm typecheck && pnpm test` passes.
- [ ] `./scripts/build-server.sh` produces a server image.
- [ ] Server starts with a generated config and applies migrations.
- [ ] Registration, login, channel creation, messaging, reactions, DM,
  WebSocket, file upload, and logout smoke tests pass.
- [ ] Desktop client starts, authenticates, sends messages, and receives
  real-time updates.
- [ ] Web UI builds, serves from the server, and supports PWA install and Web
  Push.
- [ ] Docker Compose production and build variants start and tear down cleanly.
- [ ] Domain export/import round-trip is idempotent.
- [ ] No secrets, `.env` files, or build artifacts are staged for commit.

## 10. Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| SQLx compile errors | `DATABASE_URL` not set | `source .env.testing` |
| Integration tests fail | PostgreSQL not reachable | Start the database and verify the connection string |
| Desktop client cannot connect | Wrong backend URL | Check `/settings` and `base_url` in `ruckchat.yaml` |
| Web UI not served | `web/dist/` missing or `web.path` misconfigured | `cd web && pnpm build` |
| WebSocket auth fails | Missing cookie or bearer header | Log in again; cookie is HTTP-only |
| Push notifications not received | VAPID keys not configured | Set `vapid.public_key` and `vapid.private_key` in `ruckchat.yaml` |
| Plugin command fails | Plugin not loaded or path wrong | Check `plugins.path` and server logs |

## 11. Related Documentation

- `CLAUDE.md` — implementation contract and loop.
- `server/openapi.yaml` — full REST API spec.
- `book/006-Server.md` — server conventions.
- `book/007-Desktop.md` — desktop client conventions.
- `server/README.md` — server crate guide.
- `desktop/README.md` — desktop client guide.
- `web/README.md` — Web UI guide.
- `docs/ADR-*.md` — architecture decisions.
