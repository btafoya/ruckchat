
# CLAUDE IMPLEMENTATION CONTRACT

Never change architecture without updating ADRs.

## Quick start

Generate a local configuration file and edit the database URL:

```bash
set -a
source .env.testing
set +a
cargo run -p ruckchat-server -- --init-config ./ruckchat.yaml
# edit ./ruckchat.yaml, then:
cargo run -p ruckchat-server -- --config ./ruckchat.yaml
```

Then in another terminal, run the desktop client:

```bash
cd desktop
pnpm install
pnpm tauri dev
```

## Rules

- Always fully complete the task.
- Never create stubs.
- Always build for production use.
- Always follow the `Implementation Loop` below.
- Apply the `ponytail` skill: prefer deletion over addition, reuse existing code,
  prefer stdlib/native/installed dependencies, and question whether speculative
  features need to exist at all.

## Claude Code Behaviour Guidelines

- Avoid ownership-dodging behaviour: if you encounter an issue, take responsibility for it and work towards a solution instead of passing it on to someone else. Don't say things like "not caused by my changes" or say that it's "a pre-existing issue". Instead, acknowledge the problem and take initiative to fix it. Also, don't give up with excuses like "known limitation" and don't mark it for "future work".
- Avoid premature stopping: if you encounter a problem, don't stop at the first obstacle. Instead, keep pushing forward and find a way to overcome it. Don't say things like "good stopping point" or "natural checkpoint". Instead, keep going until you have a complete solution.
- Avoid permission-seeking behaviour: if you have the knowledge and capability to solve a problem, push through. Don't say things like "should I continue?" or "want me to keep going?". Instead, take initiative and act towards the solution.
- Do plan multi-step approaches before acting (plan which files to read and in what order, which tools to use, etc).
- Do recall and apply project-specific conventions from CLAUDE.md files.
- Do catch your own mistakes by applying reasoning loops and self-checks, and fix them before committing or asking for help.

### Use of tools

Adhere to the following guidelines when using tools:

- Always use a **Research-First approach**: Before using any tool, conduct thorough research to understand the context and requirements. This ensures that you use the most appropriate tool for the task at hand. Never use an Edit-First approach. You should prefer making surgical edits to the codebase instead of rewriting whole files or doing large, sweeping changes.
- Use **Reasoning Loops** very frequently. Don't be lazy and skip them. Reasoning loops are essential for ensuring the quality and accuracy of your work.

### Thinking Depth

When working on tasks that require complex problem-solving, always apply the highest **level of thinking depth**.

When thinking is shallow, the model outputs to the cheapest action available. We don't want that. We don't mind consuming more tokens if it means a better output. So always apply the highest level of thinking depth.

Never reason from assumptions, always reason from the actual data. You need to read and understand the actual code, publication or documentation in order to make informed decisions. Don't rely on assumptions or guesses, as they can lead to mistakes and misunderstandings.

## Current Status

Phases 1–12 are complete. Phase 13 (Mobile/Flutter) is not yet implemented.

- Phase 1: Cargo workspace, shared crates (`ruckchat-id`, `ruckchat-common`,
  `ruckchat-config`), database migrations, and schema integration tests.
- Phase 2: Domain layer (`ruckchat-domain` crate) with entities, value objects,
  and repository traits.
- Phase 3: Service layer and SQLx repositories in `ruckchat-server`, plus
  unit-tested service logic using in-memory mocks.
- Phase 4: Axum REST API, authentication middleware/extractor, route handlers
  for all Phase 3 services, and integration tests against PostgreSQL.
- Phase 5: WebSocket server with authenticated `/websocket`, in-memory connection
  management, real-time event bus, and reaction REST endpoints.
- Phase 6: MCP server exposed on `/mcp/v1/sse` using the `rmcp` Streamable HTTP
  transport, with six tools, four `ruckchat://` resources, service-layer
  authorization, unit tests, integration tests, and OpenAPI documentation.
- Phase 7: Plugin SDK in `crates/ruckchat-plugin-sdk/`, server-side dynamic
  loading via `libloading`, `CompositeEventBus` event routing to plugins,
  `HostApi` for plugin interaction with the service layer, and a
  `POST /plugins/{plugin}/commands/{command}` slash-command endpoint.
- Phase 8: Desktop client in `desktop/` with Tauri v2, React 19, TypeScript,
  Tailwind CSS v4, and React Router v7. The `desktop/src-tauri` crate is part
  of the Cargo workspace. Features include API client + auth flow, core UI shell
  and navigation, state stores with real-time WebSocket sync, messaging (message
  history with pagination, composer with markdown preview and @mention autocomplete,
  typing indicators, reactions, file metadata attachments, thread replies, and
  unread badges), native integrations (OS notifications, tray icon with unread count,
  file dialogs, deep links for `ruckchat://`), offline resilience (draft persistence
  and failed-send retry), a configurable backend URL settings screen, packaging
  metadata, tests, and docs.
- Phase 9: Runtime YAML configuration. The server reads a single `ruckchat.yaml`
  file from a platform default path or a path supplied via `--config`. The file is
  the sole source of truth for runtime settings; no `.env` files or `RUCKCHAT_*`
  environment variable overrides are read.
- Phase 10: Browser-based Web UI that reuses `desktop/src` React code through a
  `desktop/src/platform/` abstraction layer, is served by the Rust server as
  static assets (embedded or from a configured directory), supports PWA
  install/service-worker offline caching, and adds Web Push notifications using a
  server-managed VAPID key.
- Phase 12: Migration and packaging tools. The server CLI supports versioned
  JSON domain-data export/import with idempotent `ON CONFLICT DO NOTHING`
  semantics and a dry-run mode. The repository includes a multi-stage `Dockerfile`
  using SQLx offline mode, a runtime `docker-compose.yml` with PostgreSQL 17, a
  `docker-compose.build.yml` for source builds, a `scripts/build-server.sh` helper,
  and a `.github/workflows/release.yml` workflow that publishes the server Docker
  image and builds cross-platform Tauri desktop installers on `v*` tags.
- RocketChat → RuckChat migration tool: standalone `rocketchat2ruckchat` binary
  crate in `crates/rocketchat2ruckchat/` with RocketChat and RuckChat REST clients,
  a SQLite mapping store, deterministic UUIDv5 transforms, file/emoji upload
  pipeline, dry-run, and interactive prompts.
- Mobile support (Flutter) is planned for a later phase.

## Commands

| Command | Description |
|---------|-------------|
| `cargo build --workspace` | Build all crates |
| `cargo test --workspace` | Run unit tests across all crates |
| `cargo test -p ruckchat-server` | Run server tests (requires `DATABASE_URL` for integration tests) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Run clippy with workspace lints |
| `cargo sqlx migrate run --source migrations/migrations` | Apply pending migrations |
| `cargo run -p ruckchat-server -- --config ./ruckchat.yaml` | Run the server with an explicit config file |
| `cargo run -p ruckchat-server -- --init-config [./ruckchat.yaml]` | Write a default config file and exit |
| `cargo run -p ruckchat-server -- --config ./ruckchat.yaml migrate export --output export.json` | Export a domain snapshot |
| `cargo run -p ruckchat-server -- --config ./ruckchat.yaml migrate import --input export.json` | Import a domain snapshot idempotently |
| `cargo run -p rocketchat2ruckchat -- --config migration.yaml --dry-run` | Dry-run a RocketChat → RuckChat migration |
| `cargo run -p rocketchat2ruckchat -- --config migration.yaml --apply` | Apply a RocketChat → RuckChat migration |
| `cargo run -p rocketchat2ruckchat -- --interactive` | Run the migration tool with interactive prompts |
| `cargo sqlx prepare --workspace` | Generate SQLx offline metadata for Docker builds |
| `./scripts/build-server.sh` | Build Web UI assets, refresh `.sqlx/`, and build the server Docker image |
| `./scripts/release.sh vX.Y.Z` | Automate a release: bump versions, run checks/builds, tag, sign, and push |
| `docker compose up -d` | Start the server and PostgreSQL via Docker Compose (pre-built image) |
| `docker compose -f docker-compose.build.yml up -d` | Build and start the server from source |
| `cd desktop && pnpm install` | Install desktop client dependencies |
| `cd desktop && pnpm tauri dev` | Run the desktop client in dev mode |
| `cd desktop && pnpm tauri build` | Build desktop installers |
| `cd desktop && pnpm typecheck` | Type-check the desktop client |
| `cd desktop && pnpm test` | Run desktop client unit tests |
| `cd web && pnpm install` | Install Web UI dependencies |
| `cd web && pnpm dev` | Run the Web UI dev server |
| `cd web && pnpm build` | Build the Web UI for the server to embed |
| `cd web && pnpm typecheck` | Type-check the Web UI |

### Desktop schema regeneration

When `server/openapi.yaml` changes, regenerate the TypeScript API types:

```bash
cd desktop
pnpm dlx openapi-typescript ../server/openapi.yaml -o src/api/schema.ts
```

Generate application icons before release builds:

```bash
cd desktop
pnpm tauri icon <source.png>
```

## Architecture

```text
root/
├── crates/
│   ├── ruckchat-id/        # Strongly-typed IDs
│   ├── ruckchat-common/    # Shared error type and validation utilities
│   ├── ruckchat-config/    # Configuration primitives, `AuthenticatedUser`, and runtime YAML parsing
│   ├── ruckchat-domain/    # Entities, value objects, and repository traits
│   ├── ruckchat-plugin-sdk/ # Plugin SDK trait, types, and `declare_plugin!` macro
│   └── rocketchat2ruckchat/ # Standalone RocketChat → RuckChat migration tool
├── server/                 # Service layer, SQLx repositories, HTTP, WebSocket, MCP, and plugins
│   ├── src/handlers/       # Axum route handlers and HTTP DTOs
│   ├── src/services/       # Business logic, service DTOs, and event bus trait
│   ├── src/repositories/   # SQLx repository implementations
│   ├── src/websocket/      # Connection manager, event bus implementation, handler
│   ├── src/mcp/            # MCP server, tools, resources, and SSE handler
│   ├── src/plugins/        # Plugin loader, manager, host API, and composite event bus
│   ├── src/testing.rs      # In-memory mock repositories and event bus
│   └── tests/              # Integration tests against PostgreSQL
├── migrations/             # SQLx migration crate and SQL files
├── desktop/                # Tauri v2 + React desktop client
│   ├── src/                # React + TypeScript frontend
│   │   ├── api/            # OpenAPI types, fetch client, API modules
│   │   ├── components/     # UI components (Shell, Sidebar, MessagePane,
│   │   │                     Composer, MessageItem, ThreadPane, etc.)
│   │   ├── context/        # React context providers for state stores
│   │   ├── hooks/          # State hooks, unread tracking, and WebSocket manager
│   │   ├── platform/       # Platform abstraction (desktop/web shims)
│   │   ├── App.tsx         # Router and provider tree
│   │   └── main.tsx        # Vite/Tauri entry point
│   ├── src-tauri/          # Tauri Rust shell
│   └── README.md           # Desktop developer guide
├── web/                    # Vite React web client (shares desktop/src)
│   ├── src/
│   │   ├── App.tsx         # Web entry point with web platform hooks
│   │   └── main.tsx
│   ├── public/             # PWA manifest, icons, service worker
│   ├── package.json
│   ├── vite.config.ts
│   └── README.md
├── book/                   # mdBook-style project documentation
├── docs/
│   └── ADR-*.md            # Architecture Decision Records
└── server/openapi.yaml     # Full REST API specification
```

## Key Files

- `Cargo.toml` — Workspace manifest with shared dependencies and strict lints.
- `server/src/lib.rs` — Server crate entry point and `connect_database` helper.
- `server/src/services/` — Business logic and DTOs.
- `server/src/repositories/` — SQLx implementations of domain repository traits.
- `server/src/handlers/` — Axum route handlers, authentication extractor, and HTTP DTOs.
- `server/src/websocket/` — WebSocket connection manager, event bus, and upgrade handler.
- `server/src/services/mcp.rs` — MCP service bridge that delegates to the existing service layer.
- `server/src/mcp/` — MCP server handler, tools, resources, and Streamable HTTP handler.
- `server/src/plugins/loader.rs` — Dynamic library loading and API-version validation.
- `server/src/plugins/manager.rs` — Plugin lifecycle, event dispatch, and command routing.
- `server/src/plugins/host.rs` — `HostApi` implementation that bridges plugins to services.
- `server/src/plugins/bus.rs` — `CompositeEventBus` that routes events to both WebSocket clients and plugins.
- `server/src/testing.rs` — In-memory mock repositories and event bus for service unit tests.
- `desktop/src-tauri/` — Tauri v2 Rust shell and native integrations.
- `desktop/src/` — React + TypeScript desktop UI.
- `desktop/src/components/MessagePane.tsx` — Message list, reactions, typing
  indicator, and thread pane host.
- `desktop/src/components/Composer.tsx` — Message composer with markdown preview,
  @mention autocomplete, file attachments, and typing WebSocket messages.
- `desktop/src/components/ThreadPane.tsx` — Thread reply detail pane.
- `desktop/src/components/MessageItem.tsx` — Individual message with reactions
  and reply action.
- `desktop/src/hooks/useMessages.ts` — Message history, send, failed-send retry,
  reactions cache, and thread reply loading.
- `desktop/src/hooks/useUnread.ts` — Local unread counts driven by WebSocket
  events.
- `desktop/src/hooks/useSettings.ts` — Configurable backend URL and notification
  preference, persisted in `localStorage`.
- `desktop/src/hooks/useNotifications.ts` — OS notification permission and
  delivery for mentions and DMs.
- `desktop/src/hooks/useTray.ts` — Reflects the total unread count in the tray
  tooltip.
- `desktop/src/hooks/useDeepLink.ts` — Reads the current `ruckchat://` deep-link
  URL on startup.
- `desktop/src/components/Settings.tsx` — Backend URL and notification settings
  screen.
- `desktop/src-tauri/src/lib.rs` — Tray setup, `set_unread_count`,
  `get_deep_link_url`, and plugin initialization.
- `server/tests/` — Integration tests against PostgreSQL.
- `server/tests/mcp.rs` — MCP Streamable HTTP endpoint integration tests.
- `server/tests/migrate.rs` — Domain snapshot export/import integration tests.
- `migrations/migrations/` — SQLx `.up.sql` / `.down.sql` migration files.
- `server/openapi.yaml` — Full REST API specification for the REST API, WebSocket upgrade, and MCP endpoint.
- `Dockerfile` — Multi-stage SQLx-offline server image build.
- `docker-compose.yml` — PostgreSQL 17 + server orchestration.
- `scripts/build-server.sh` — Build Web UI assets, refresh `.sqlx/`, and build the Docker image.
- `.github/workflows/release.yml` — Cross-platform Tauri desktop installer releases on `v*` tags.
- `docs/ADR-003-Shared-Crates.md`, `docs/ADR-004-Migrations.md`,
  `docs/ADR-005-Domain-Crate.md`, `docs/ADR-006-WebSocket-Real-Time-Events.md`,
  `docs/ADR-007-MCP-Server.md`, `docs/ADR-008-Desktop-Client.md`,
  `docs/ADR-009-Plugin-SDK.md`, `docs/ADR-010-Runtime-YAML-Configuration.md`,
  `docs/ADR-011-Web-UI.md`, `docs/ADR-012-Migration-and-Packaging.md` — Active ADRs.

## Environment

Required at **compile time** for SQLx query verification in the server crate:
- `DATABASE_URL` — PostgreSQL connection string, e.g.
  `postgres://ruckchat:ruckchat@localhost/ruckchat`.

A local `.env.testing` file is provided at the repo root with this value.
Source it before workspace checks:

```bash
set -a
source .env.testing
set +a
```

At **runtime** the server reads a single YAML configuration file:
- Default path: `/etc/ruckchat/ruckchat.yaml` (Linux),
  `/Library/Application Support/RuckChat/ruckchat.yaml` (macOS), or
  `%ProgramData%\RuckChat\ruckchat.yaml` (Windows).
- Override with `--config <path>`.
- Generate a template with `ruckchat-server --init-config [path]`.

Required for schema/migration tests that create isolated per-test databases:
- `RUCKCHAT_TEST_ADMIN_DATABASE_URL` — Admin connection string used to create and
  drop temporary test databases, e.g.
  `postgres://postgres:postgres@localhost:5445/postgres`.

The server does **not** read `.env` files or `RUCKCHAT_*` environment variables
at runtime. All runtime settings live in `ruckchat.yaml`.

## Testing

- `cargo test --workspace` runs unit tests without a database.
- `ruckchat-server` integration tests require a running PostgreSQL database and
  `DATABASE_URL`. `connect_database` applies pending migrations on startup.
- Schema/migration tests in `migrations/tests/schema.rs` require
  `RUCKCHAT_TEST_ADMIN_DATABASE_URL` and create isolated databases for each test.
- Services are unit-tested against in-memory mocks in `server/src/testing.rs`,
  not against the real database.
- MCP integration tests exercise the `/mcp/v1/sse` Streamable HTTP endpoint,
  including initialization, tool calls, and resource reads.
- Desktop unit and component tests live in `desktop/src/**/*.test.tsx` and are run
  with `pnpm test` inside the `desktop/` directory.

## CodeGraph and MCP Tooling

Use the [CodeGraph MCP server](https://colbymchenry.github.io/codegraph/getting-started/introduction/)
for structural questions. Prefer `codegraph_explore` over `grep` or chained `Read`
calls; trust its AST-parsed results. Use other configured MCP servers when they
provide a dedicated tool for the task.

## Implementation Loop

Every implementation task must follow this sequence and stop at the first
step that does not pass. Do not skip steps, and do not commit code that has
not passed every applicable check.

```
Read docs
    ↓
Plan
    ↓
Write code
    ↓
cargo fmt
    ↓
cargo check
    ↓
cargo clippy
    ↓
cargo nextest
    ↓
Fix
    ↓
Update docs
    ↓
Commit
    ↓
Update codegraph `codegraph index`
```

### Read docs

Read ADRs first, then `book/000-Vision.md` through `book/006-Server.md` as the
task touches them. Check `.claude/plan.md` for active phase plans.

### Plan

State assumptions, identify affected crates/services/repositories, and decide if
an ADR needs updating before code changes.

### Write code

Follow **Implementation Order**, keep changes surgical, and match existing style.

### Format, check, lint, test

| Step | Command | Stop if it fails |
|------|---------|------------------|
| Format | `cargo fmt --all` | Yes |
| Check | `cargo check --workspace` | Yes |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Yes |
| Test | `cargo nextest run --workspace` | Yes |
| Type check (desktop) | `cd desktop && pnpm typecheck` | Yes |
| Unit tests (desktop) | `cd desktop && pnpm test` | Yes |

If `cargo nextest` is not installed, use `cargo test --workspace` as a fallback.

### Update docs

Update `server/openapi.yaml`, `book/*.md`, `docs/ADR-*.md`, `server/README.md`,
`desktop/README.md`, and this `CLAUDE.md` as the change touches them.

### Commit

- Author commits as `Brian Tafoya <btafoya@briantafoya.com>`.
- Do not include AI attribution in commit messages or code comments.
- Never commit `.env` files or secrets.

### Update codegraph

After committing, refresh the CodeGraph index so future structural queries reflect the new code:

```bash
codegraph index
```

Or use the equivalent CodeGraph MCP server action.

## Gotchas

- Workspace lints are strict (`workspace.lints.rust` and `workspace.lints.clippy`
  in `Cargo.toml`). `cargo clippy` must pass with `-D warnings`.
- `cargo nextest` is the default test runner in the implementation loop; install
  with `cargo install cargo-nextest` if it is not present.
- `cargo build`, `cargo check`, and `cargo clippy` require `DATABASE_URL` because
  the server crate uses SQLx online query macros (`sqlx::query!`).
- `server/src/main.rs` starts the full Axum HTTP server with WebSocket and MCP
  support when enabled.
- `migrations` is a Cargo workspace member, not just a directory of SQL files.
- Repository traits live in `ruckchat-domain`; SQLx implementations live in
  `server/src/repositories/`.

- The desktop client defaults to `http://localhost:3000` for development and
  exposes a settings screen to change the backend URL. The chosen URL is stored
  in `localStorage` and used by all API calls and the WebSocket connection.
  WebSocket authentication relies on the HTTP-only `ruckchat_session` cookie
  set at login; restoring from `localStorage` alone is not sufficient.

## Implementation Order

1. Cargo workspace → 2. Shared crates → 3. Database schema → 4. Domain layer →
5. Services → 6. REST API → 7. WebSocket server → 8. MCP server → 9. Plugin SDK →
10. Desktop → 11. Runtime YAML configuration → 12. Web UI → 13. Migration and
    packaging tools → 14. Mobile.

Ship unit tests, integration tests, OpenAPI updates, and docs with every feature.
