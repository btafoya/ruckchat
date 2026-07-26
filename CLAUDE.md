
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

- Never use `http://localhost:8322/` as the url to access the webui, use `https://ruck.premadev.com/` instead
- Always fully complete the task.
- Never create stubs.
- Always build for production use.
- Always follow the `Implementation Loop` below.
- Apply the `ponytail` skill: prefer deletion over addition, reuse existing code,
  prefer stdlib/native/installed dependencies, and question whether speculative
  features need to exist at all.
- Runtime config lives in ruckchat.yaml, not .env. The .env.testing file is
  only for compile-time SQLx query verification (DATABASE_URL), which is separate from
  runtime settings.

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

Phases 1–12 and Phase 14 (Web UI Admin Panel) are complete. Phase 13 (Mobile/Flutter) is not yet implemented.

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
  history with pagination, composer with markdown preview, a formatting toolbar
  (bold/italic/strike/code/lists/blockquote/code block), inline image insertion
  via a picker that uploads through the file service and embeds a
  `/files/{file_id}/content` URL, and @mention autocomplete, typing indicators,
  reactions, file metadata attachments, thread replies, and unread badges),
  native integrations (OS notifications, tray icon with unread count,
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
  install/service-worker offline caching with a network-first app-shell
  strategy, and adds Web Push notifications using a server-managed VAPID key.
- Phase 12: Migration and packaging tools. The server CLI supports versioned
  JSON domain-data export/import with idempotent `ON CONFLICT DO NOTHING`
  semantics and a dry-run mode. The repository includes a multi-stage `Dockerfile`
  using SQLx offline mode, a runtime `docker-compose.yml` with PostgreSQL 17, a
  `docker-compose.build.yml` for source builds, and a `scripts/publish.sh`
  helper that builds the server Docker image and publishes releases.
- RocketChat → RuckChat migration tool: standalone `rocketchat2ruckchat` binary
  crate in `crates/rocketchat2ruckchat/` with RocketChat and RuckChat REST clients,
  a SQLite mapping store, deterministic UUIDv5 transforms, file/emoji upload
  pipeline, dry-run, and interactive prompts.
- `scripts/publish.sh vX.Y.Z` automates the full release pipeline: version
  bumps, CHANGELOG generation, validation checks, local server and desktop
  builds, GPG-signed commit/tag/push, and publishing the Docker image to
  GHCR and the GitHub Release with all built assets. Replaces the former
  `scripts/release.sh` / `scripts/build-server.sh` split and the
  `.github/workflows/release.yml` CI publish job, which is retired; the
  legacy scripts are kept as `scripts/release-old.sh` /
  `scripts/build-server-old.sh` for reference.
- Phase 14 (Web UI Admin Panel): server-wide `users.is_server_admin` flag,
  database-backed `server_settings` with YAML override precedence, append-only
  `audit_log`, server-admin impersonation, server admin REST endpoints under
  `/api/v1/server/*`, org admin additions under `/api/v1/admin/organizations/{id}/*`,
  OpenAPI updates, backend integration tests, and shared React admin components
  with routes in `desktop/src/PlatformShell.tsx`.
- Composer/message-format issue work (`docs/issues/WORKFLOW.md` Phase 2):
  @mentions as first-class Tiptap nodes with `mentioned_user_ids` on messages,
  and a server-side spell-checker. The `crates/ruckchat-spelling` crate embeds
  a Hunspell `en-US` dictionary via the pure-Rust `spellbook` crate; the
  `SpellingService` rate-limits `POST /api/v1/spelling/check`,
  `POST /api/v1/spelling/suggest`, and `GET /api/v1/spelling/languages`;
  `desktop/src/spelling/SpellingProofreader.ts` wires
  `@farscrl/tiptap-extension-spellchecker` into the shared composer. Gated by
  the `spelling_enabled` / `spelling_default_language` server settings.
- Conversation discovery issue work (`docs/issues/WORKFLOW.md` Phase 3):
  single-organization auto-redirect to the last-selected channel or
  `#general` (`OrgIndexRoute` / `ChannelIndexRoute` and
  `desktop/src/lastConversation.ts`), channel creation/management for any
  organization member with creator/manager edit and archive rights, and
  direct-message start/list/hide-reappear via
  `desktop/src/components/StartDmModal.tsx`.
- Admin UI polish issue work (`docs/issues/WORKFLOW.md` Phase 4): back-to-chat
  links in `ServerAdminShell.tsx` and `OrgAdminShell.tsx`; a collapsible
  mobile nav for `OrgAdminShell.tsx`; a complete `OrgAdminMembers.tsx`
  (invite/list/role-change/remove); per-team member and room management in
  `OrgAdminTeams.tsx` backed by new
  `/api/v1/admin/organizations/{id}/teams/{team_id}/members` and `/rooms`
  endpoints; real file upload in `OrgAdminEmoji.tsx`; and
  `desktop/src/components/admin/EditUserModal.tsx`, a combined create/edit
  user modal with promote/demote, password reset, deactivate/reactivate, and
  a danger-zone permanent delete backed by a new
  `DELETE /api/v1/server/users/{user_id}` endpoint (`UserRepository::delete`,
  guarded against deleting the last server admin and against users with
  existing message/organization-ownership history via a foreign-key-violation
  → `409 Conflict` mapping).
- Home and user-profile issue work (`docs/issues/WORKFLOW.md` Phase 6):
  `/org` now renders an `OrgIndex` picker/empty-state view (multi-org or zero-org)
  and redirects a single-organization user to `/org/{id}`; the org home route
  (`desktop/src/components/OrgHome.tsx`) lists non-archived channels and direct
  messages for that organization, sorted by unread count descending, with links
  into each conversation and unread badges driven by `ReadStateContext`. The
  user's theme preference (`light`/`dark`/`system`) is now stored in the server
  profile (`users.theme`, `User::theme`, `PATCH /api/v1/users/me`) and loaded on
  login/registration/profile restore; `Settings.tsx` persists theme changes to
  the server (`docs/ADR-017-Server-Stored-Theme-Preference.md`).
- Search and read-state issue work (`docs/ADR-015-Search-And-Read-State.md`):
  message editing wired into the desktop client (`MessagesApi.edit`, a
  Composer edit mode, an author-only "Edit" action) on top of the already-
  existing backend edit/broadcast path; global search across messages,
  channels, people, and files at `GET /organizations/{id}/search` with
  Gmail-style `from:`/`in:`/`has:attachment`/`before:`/`after:`/`is:unread`
  operators parsed server-side (`server/src/services/search.rs`); and a new
  server-side, per-message read-state model (`message_reads` table,
  `ReadStateService`, `POST /channels/{id}/read`,
  `POST /direct_messages/{id}/read`, `GET /organizations/{id}/unread_counts`,
  the `read_state.updated` WebSocket event) that replaced the old
  `localStorage`-only `useUnread.ts` outright.
- File attachment visibility fix: `Message` (`crates/ruckchat-domain/src/message.rs`)
  gained an `attachments: Vec<File>` field, populated by
  `MessageRepositorySqlx` (`server/src/repositories/message.rs`) via a batched
  `message_files`/`files` join in `by_id`, `list_by_conversation`, and
  `search`. `FileService::attach_file_to_message`
  (`server/src/services/file.rs`) now publishes a `message.updated` event
  after attaching so open clients see the attachment without a refresh.
  `MessageItem.tsx` renders attachments as download links. The actual root
  cause of attachments never appearing: `POST /messages/{id}/attachments`
  (`server/src/handlers/file.rs`) deserialized its JSON body into the
  service-layer `AttachFileRequest`, which required a `message_id` field the
  real client (and the documented OpenAPI schema) never sent — every real
  attach call 422'd and was silently swallowed by a `console.warn` in
  `useMessages.ts`. Fixed by extracting a body-only `AttachFileBody { file_id
  }` and building the service DTO from the path param instead. Regression
  test: `attach_file_to_message_shows_up_in_history` in `server/tests/file.rs`.
- Message list reload (`docs/ADR-016-Cursor-Based-Message-Pagination.md`):
  replaced offset-based (`LIMIT/OFFSET`) message history/thread-reply
  pagination with keyset pagination on a `(created_at, id)` cursor
  (`MessageId` is a random `Uuid::new_v4()`, not time-sortable, so `id`
  alone can't be a cursor). `MessageRepository::list_before`/`list_after`/
  `list_replies_before`/`list_replies_after` replace `list_by_conversation`;
  `MessageService::get_history`/`get_thread_replies` take a
  `before_id`/`after_id`/`around_id`/`limit` `MessagePageQuery` and return a
  `MessagePage` with `has_more_older`/`has_more_newer`. HTTP responses wrap
  each message in `MessageWithReadState` (`is_unread: bool`, computed
  per-caller at the handler layer via a new `ReadStateService::unread_ids`
  passthrough — never on the shared domain `Message`, which is also the
  WebSocket broadcast/MCP payload). The desktop/web `useMessages.ts` was
  rewritten around this: ascending order end-to-end, `loadOlder`/
  `loadNewer`/`jumpToMessage`, an explicit live-tail-vs-anchored-history
  mode (`hasMoreNewer`) that gates whether WebSocket `message.created`
  events get spliced into the loaded window, and a local unread-id `Set`
  cleared via the new `useMarkReadBatcher` hook. `MessagePane.tsx`/
  `ThreadPane.tsx` gained scroll-anchored automatic scroll-up pagination
  (`IntersectionObserver` top sentinel), auto-follow with a "↓ N new
  messages" pill when scrolled up, per-message unread dots
  (`MessageItem.tsx`) that clear on scroll-into-view, and a `?message=<id>`
  deep-link (wired from `SearchResultsPage.tsx`) that anchors and highlights
  a specific message. `search.rs` and the MCP `get_messages`/
  `search_messages` tools are unaffected. Integration tests:
  `server/tests/message_pagination.rs`.
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
| `./scripts/publish.sh --dry-run vX.Y.Z` | Print the full release plan without executing it |
| `./scripts/publish.sh vX.Y.Z` | Full release: bump versions, run checks/builds, tag, sign, push, and publish to GHCR/GitHub |
| `./scripts/publish.sh --no-confirm vX.Y.Z-rc.1` | Full release without interactive confirmation prompts |
| `./scripts/publish.sh --build-only` | Build Web UI assets, refresh `.sqlx/`, and build the server Docker image/`.deb`/desktop bundles only |
| `./scripts/publish.sh --publish-only vX.Y.Z` | Publish previously built artifacts (Docker image, `.deb`, desktop bundles) without rebuilding |
| `./scripts/server.sh start` | Start the server and PostgreSQL via Docker Compose (pre-built image; always recreates) |
| `./scripts/server.sh start --build` | Rebuild the server image from source (`docker-compose.build.yml`) and start it; always recompiles, even if a `root-server` image already exists |
| `./scripts/server.sh stop` | Stop and remove the Docker Compose stack |
| `./scripts/server.sh stop --keep` | Stop containers but keep state for a fast restart |
| `./scripts/server.sh restart` | Stop then start the Docker Compose stack |
| `./scripts/server.sh status` | Show running Docker Compose containers |
| `./scripts/server.sh logs` | Follow Docker Compose container logs |
| `docker compose up -d` | Start the server and PostgreSQL via Docker Compose directly (pre-built image) |
| `docker compose -f docker-compose.build.yml up -d --build` | Build and start the server from source directly (the `--build` flag is required; without it, `up` reuses any existing local image and silently ignores source changes) |
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
│   ├── ruckchat-spelling/  # Embedded Hunspell spelling engine (`spellbook`-backed)
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
  a formatting toolbar, an image picker that uploads via `FilesApi.uploadFile`
  and inserts a `/files/{file_id}/content` image node, @mention autocomplete,
  file attachments, and typing WebSocket messages.
- `desktop/src/components/ThreadPane.tsx` — Thread reply detail pane.
- `desktop/src/components/MessageItem.tsx` — Individual message with reactions
  and reply action.
- `desktop/src/hooks/useMessages.ts` — Cursor-paginated message history,
  send/retry, reactions, thread replies, live-tail anchoring, and `?message=<id>`
  deep-link jumping.
- `desktop/src/hooks/useReadState.ts` — Server-backed unread counts and
  read-state API.
- `desktop/src/context/ReadStateContext.tsx` — Shared read-state instance for
  `Sidebar`/`PlatformShell`.
- `desktop/src/hooks/useMarkReadBatcher.ts` — Batches scroll-into-view
  read-state calls.
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
- `desktop/src/components/OrgHome.tsx` — Organization home view listing
  unread-sorted channels and direct messages with links into each conversation.
- `desktop/src/components/OrgIndex.tsx` — Organization picker for multi-org
  users and empty state for users with no organizations.
- `desktop/src/PlatformShell.tsx` — Authenticated router shell; redirects
  single-org `/org` to `/org/{id}` and renders `OrgHome` as the org route.
- `crates/ruckchat-domain/src/user.rs` — User aggregate with server-stored
  `theme` preference.
- `desktop/src-tauri/src/lib.rs` — Tray setup, `set_unread_count`,
  `get_deep_link_url`, and plugin initialization.
- `server/tests/` — Integration tests against PostgreSQL.
- `server/tests/mcp.rs` — MCP Streamable HTTP endpoint integration tests.
- `server/tests/migrate.rs` — Domain snapshot export/import integration tests.
- `server/tests/server_admin.rs` — Server admin endpoint integration tests.
- `server/src/services/server_admin.rs` — Server admin, impersonation, and admin user operations.
- `server/src/services/audit.rs` — Append-only audit log writer.
- `server/src/services/server_settings.rs` — Database settings with YAML override merge.
- `server/src/handlers/server_admin.rs` — Server admin and impersonation REST handlers.
- `desktop/src/api/serverAdmin.ts` and `desktop/src/api/orgAdmin.ts` — Admin API clients.
- `desktop/src/components/admin/*.tsx` — Server and org admin React components.
- `desktop/src/components/Sidebar.tsx` — Admin navigation links gated by role.
- `crates/ruckchat-spelling/src/lib.rs` — Embedded Hunspell `SpellingEngine`.
- `server/src/services/spelling.rs` — Rate-limited spell-checker service.
- `server/src/handlers/spelling.rs` — Spell-checker REST handlers.
- `server/tests/spelling.rs` — Spell-checker endpoint integration tests.
- `desktop/src/spelling/SpellingProofreader.ts` — `IProofreaderInterface` implementation calling the spelling REST endpoints.
- `desktop/src/api/spelling.ts` — Spelling REST API client.
- `server/src/services/read_state.rs` — Per-message read-state service (`message_reads` table).
- `server/src/services/search.rs` — Cross-content-type search service and Gmail-style `parse_query`.
- `server/src/handlers/search.rs` — Global search REST handler.
- `server/tests/search_and_read_state.rs` — Search and read-state integration tests.
- `desktop/src/hooks/useReadState.ts` — Server-backed unread badges (replaces the removed `useUnread.ts`).
- `desktop/src/context/ReadStateContext.tsx` — Shared read-state instance for `Sidebar`/`PlatformShell`.
- `desktop/src/components/SearchResultsPage.tsx` — Global search results route.
- `desktop/src/api/messages.ts`, `desktop/src/api/search.ts` — Message-edit and search REST clients.
- `crates/ruckchat-domain/src/repositories.rs` — `MessageCursor` and the `MessageRepository` keyset-pagination methods.
- `server/tests/message_pagination.rs` — Cursor pagination and `is_unread` integration tests.
- `desktop/src/hooks/useMarkReadBatcher.ts` — Batches scroll-into-view read-state calls.
- `migrations/migrations/` — SQLx `.up.sql` / `.down.sql` migration files.
- `server/openapi.yaml` — Full REST API specification for the REST API, WebSocket upgrade, and MCP endpoint.
- `Dockerfile` — Multi-stage SQLx-offline server image build.
- `docker-compose.yml` — PostgreSQL 17 + server orchestration.
- `scripts/server.sh` — Start, stop, restart, and inspect the Docker Compose stack.
- `scripts/publish.sh` — Flag-driven release pipeline: bump versions, run
  checks/builds, generate CHANGELOG, GPG-sign commit/tag/push, build the
  Docker image, `.deb` package, and desktop bundles, and publish to GHCR and
  GitHub Releases.
- `docs/ADR-003-Shared-Crates.md`, `docs/ADR-004-Migrations.md`,
  `docs/ADR-005-Domain-Crate.md`, `docs/ADR-006-WebSocket-Real-Time-Events.md`,
  `docs/ADR-007-MCP-Server.md`, `docs/ADR-008-Desktop-Client.md`,
  `docs/ADR-009-Plugin-SDK.md`, `docs/ADR-010-Runtime-YAML-Configuration.md`,
  `docs/ADR-011-Web-UI.md`, `docs/ADR-012-Migration-and-Packaging.md`,
  `docs/ADR-013-Web-UI-Admin-Panel.md`, `docs/ADR-014-Spell-Checker.md`,
  `docs/ADR-015-Search-And-Read-State.md`,
  `docs/ADR-016-Cursor-Based-Message-Pagination.md`,
  `docs/ADR-017-Server-Stored-Theme-Preference.md` — Active ADRs.

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
- **Web UI e2e testing (Playwright)**: `cd web && pnpm test:e2e` runs
  `web/tests/*.spec.ts` against **https://ruck.premadev.com** — this domain is
  a **local dev environment** (not a production deployment with real user
  data), so it's safe for tests to register throwaway accounts, send
  messages, and create/archive channels against it. Each spec self-registers
  a uniquely-named account/organization, so specs don't collide with each
  other or with prior runs. Override the target with `RUCKCHAT_E2E_BASE_URL`.
  The admin-CRUD spec additionally needs `RUCKCHAT_E2E_ADMIN_EMAIL` /
  `RUCKCHAT_E2E_ADMIN_PASSWORD` (an existing server-admin account on that
  instance) and skips itself when they're unset, since there's no "first
  user" to auto-promote on an already-populated instance. First-time setup:
  `cd web && pnpm test:e2e:install` to fetch the Chromium binary.

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
Rebuild Docker stack (if verifying via Docker)
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

### Rebuild Docker stack (if verifying via Docker)

`docker-compose.yml` runs a fixed, pre-built image tag with no source
bind-mount, so a plain `docker compose up -d` / `./scripts/server.sh start`
never reflects local edits — it only recreates containers from whatever
image was last pulled or built. To manually verify a change against the
Docker stack, rebuild from the current source tree and recreate the
containers:

```bash
./scripts/server.sh start --build
```

This uses `docker-compose.build.yml` and always passes `--build` to `docker
compose up`, so it recompiles even if a `root-server` image already exists
locally. If the server still fails at startup with a migration or schema
error after this (a stale BuildKit layer cache reusing an old `COPY
migrations`/`COPY crates` layer), force a clean rebuild:

```bash
docker compose -f docker-compose.build.yml build --no-cache server
./scripts/server.sh start --build
```

`docker-compose.yml` and `docker-compose.build.yml` must keep the same
`ports:` container-side value (right side of `HOST:CONTAINER`), matching the
port in `base_url` inside `ruckchat.yaml` — see the Gotchas section.

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

## Release Workflow

Use `scripts/publish.sh` to create a release:

```bash
./scripts/publish.sh --dry-run v0.2.0
./scripts/publish.sh v0.2.0
```

The script expects a GPG signing key (`git config user.signingkey`) and a
`gh` CLI authenticated with `gh auth login`; it operates on `origin/main`.
It will:

1. Validate the requested `vx.x.x` (or `vx.x.x-<prerelease>`) tag.
2. Bump versions in `Cargo.toml`, `desktop/package.json`, `web/package.json`,
   and `desktop/src-tauri/tauri.conf.json`.
3. Generate a `CHANGELOG.md` entry from commits since the last tag.
4. Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features`,
   and `cargo test --workspace`.
5. Build the server Docker image and a `cargo-deb` `.deb` package.
6. Build desktop Tauri bundles for the host OS and cross-compile for
   Windows/macOS when the Rust targets are installed.
7. Commit and GPG-sign as `Brian Tafoya <btafoya@briantafoya.com>`, create an
   annotated GPG-signed tag, and push both to `origin/main`.
8. Push the Docker image to GHCR (`:VERSION` and `:latest`) and create the
   GitHub Release, uploading the server `.deb` and every desktop bundle built.

Other flags: `--build-only` (steps 5–6 only, no bump/commit/tag/publish),
`--publish-only` (steps 8 only, for retrying a failed publish against
already-built artifacts and an already-tagged version), `--no-build`,
`--no-publish`, `--no-checks`, `--no-bump`, `--no-desktop`, and `--no-confirm`.
Run `./scripts/publish.sh --help` for the full list.

There is no CI publish job — `scripts/publish.sh` is the sole release
pipeline and publishes GHCR and GitHub Release artifacts directly from the
machine it runs on.

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

- The server binds to the port declared in `base_url` inside `ruckchat.yaml`.
  When using Docker Compose, the **container target port** (right side of the
  `ports:` mapping) must match that port in *both* `docker-compose.yml` and
  `docker-compose.build.yml` — they must be kept in sync manually; nothing
  enforces it. `scripts/server.sh start` warns if the active compose file
  disagrees with `ruckchat.yaml`.

- `docker-compose.yml`'s `server` service runs a pinned image tag with no
  source bind-mount. Local code changes are invisible to it until an image is
  rebuilt and the container recreated — `docker compose up -d` alone will
  never pick them up. Use `./scripts/server.sh start --build` (source build
  via `docker-compose.build.yml`, forces `docker compose up --build`) instead.
  See "Rebuild Docker stack" in the Implementation Loop.

- Rebuilding via `docker-compose.build.yml` migrates the database to the
  current schema but leaves the plain `ruckchat-server:latest` tag untouched.
  Restarting afterward with `./scripts/server.sh start` (no `--build`) or a
  bare `docker compose up -d` switches back to that stale image, which can be
  *older* than the now-migrated database and fails at startup with
  `migration <version> was previously applied but is missing in the resolved
  migrations` — the mirror image of the missing-migration error the stale
  image itself causes when it's the older side. After a source-build restart
  that applied new migrations, run `./scripts/publish.sh --build-only vX.Y.Z`
  to refresh `ruckchat-server:latest` too before switching back to the plain
  path (`build_server_image` always tags both `ruckchat-server:${VERSION}`
  and `ruckchat-server:latest` locally).

- WebSocket event payload tags: the shared `ServerEvent` enum uses serde
  `rename_all = "snake_case"`, so emitted JSON tags are `message_created`,
  `reaction_added`, etc. The Rust envelope's `event_type()` returns
  dot-notation strings (`message.created`, `reaction.added`). Client-side
  switch statements must match the serde tag (`message_created`), not the
  envelope string, or live events will be silently dropped.

- The desktop client defaults to `http://localhost:3000` for development and
  exposes a settings screen to change the backend URL. The chosen URL is stored
  in `localStorage` and used by all API calls and the WebSocket connection.
  WebSocket authentication relies on the HTTP-only `ruckchat_session` cookie
  set at login; restoring from `localStorage` alone is not sufficient.

## Implementation Order

1. Cargo workspace → 2. Shared crates → 3. Database schema → 4. Domain layer →
5. Services → 6. REST API → 7. WebSocket server → 8. MCP server → 9. Plugin SDK →
10. Desktop → 11. Runtime YAML configuration → 12. Web UI → 13. Migration and
    packaging tools → 14. Web UI Admin Panel → 15. Mobile.

Ship unit tests, integration tests, OpenAPI updates, and docs with every feature.
