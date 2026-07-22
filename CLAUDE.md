
# CLAUDE IMPLEMENTATION CONTRACT

Never change architecture without updating ADRs.

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

Phases 1–6 are complete. Phase 8 (Desktop client) is in progress; Phase 7 and
Phases 9–12 are not yet implemented.

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
- Phase 8 (in progress): Desktop client in `desktop/` with Tauri v2, React 19,
  TypeScript, Tailwind CSS v4, and React Router v7. The `desktop/src-tauri`
  crate is part of the Cargo workspace. Completed so far: API client + auth
  flow, core UI shell and navigation, and state stores with real-time WebSocket
  sync. Remaining: messaging features (composer, reactions, attachments,
  threads), native integrations, offline behavior, packaging, and docs.
- Plugin and mobile support are added in later phases.

## Commands

| Command | Description |
|---------|-------------|
| `cargo build --workspace` | Build all crates |
| `cargo test --workspace` | Run unit tests across all crates |
| `cargo test -p ruckchat-server` | Run server tests (requires `DATABASE_URL` for integration tests) |
| `cargo clippy --workspace -- -D warnings` | Run clippy with workspace lints |
| `cargo sqlx migrate run --source migrations/migrations` | Apply pending migrations |
| `cargo run -p ruckchat-server` | Run the server binary (HTTP, WebSocket, MCP) |
| `cd desktop && pnpm install` | Install desktop client dependencies |
| `cd desktop && pnpm tauri dev` | Run the desktop client in dev mode |
| `cd desktop && pnpm tauri build` | Build desktop installers |
| `cd desktop && pnpm typecheck` | Type-check the desktop client |
| `cd desktop && pnpm test` | Run desktop client unit tests |

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
│   ├── ruckchat-config/    # Configuration primitives and `AuthenticatedUser`
│   └── ruckchat-domain/    # Entities, value objects, and repository traits
├── server/                 # Service layer, SQLx repositories, HTTP, WebSocket, and MCP
│   ├── src/handlers/       # Axum route handlers and HTTP DTOs
│   ├── src/services/       # Business logic, service DTOs, and event bus trait
│   ├── src/repositories/   # SQLx repository implementations
│   ├── src/websocket/      # Connection manager, event bus implementation, handler
│   ├── src/mcp/            # MCP server, tools, resources, and SSE handler
│   ├── src/testing.rs      # In-memory mock repositories and event bus
│   └── tests/              # Integration tests against PostgreSQL
├── migrations/             # SQLx migration crate and SQL files
├── desktop/                # Tauri v2 + React desktop client
│   ├── src/                # React + TypeScript frontend
│   │   ├── api/            # OpenAPI types, fetch client, API modules
│   │   ├── components/     # UI components (Shell, Sidebar, MessagePane, etc.)
│   │   ├── context/        # React context providers for state stores
│   │   ├── hooks/          # State hooks and WebSocket connection manager
│   │   ├── App.tsx         # Router and provider tree
│   │   └── main.tsx        # Vite/Tauri entry point
│   ├── src-tauri/          # Tauri Rust shell
│   └── README.md           # Desktop developer guide
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
- `server/src/testing.rs` — In-memory mock repositories and event bus for service unit tests.
- `desktop/src-tauri/` — Tauri v2 Rust shell and native integrations.
- `desktop/src/` — React + TypeScript desktop UI.
- `server/tests/` — Integration tests against PostgreSQL.
- `server/tests/mcp.rs` — MCP Streamable HTTP endpoint integration tests.
- `migrations/migrations/` — SQLx `.up.sql` / `.down.sql` migration files.
- `server/openapi.yaml` — Full OpenAPI specification for the REST API, WebSocket upgrade, and MCP endpoint.
- `docs/ADR-003-Shared-Crates.md`, `docs/ADR-004-Migrations.md`,
  `docs/ADR-005-Domain-Crate.md`, `docs/ADR-006-WebSocket-Real-Time-Events.md`,
  `docs/ADR-007-MCP-Server.md`, `docs/ADR-008-Desktop-Client.md` — Active ADRs.

## Environment

Required for integration tests and the running server:
- `DATABASE_URL` — PostgreSQL connection string, e.g.
  `postgres://ruckchat:ruckchat@localhost/ruckchat`.

A local `.env.testing` file is provided at the repo root with these values.
Source it before workspace checks:

```bash
set -a
source .env.testing
set +a
```

Required for schema/migration tests that create isolated per-test databases:
- `RUCKCHAT_TEST_ADMIN_DATABASE_URL` — Admin connection string used to create and
  drop temporary test databases, e.g.
  `postgres://postgres:postgres@localhost:5445/postgres`.

Optional via `ruckchat.toml` or `RUCKCHAT_*` environment variables:
- `RUCKCHAT_APP_NAME`
- `RUCKCHAT_ENVIRONMENT`
- `RUCKCHAT_BASE_URL`
- `RUCKCHAT_LOG_LEVEL`
- `RUCKCHAT_MCP_ENABLED` — Enable the MCP endpoint (default `true`).
- `RUCKCHAT_MCP_REQUIRE_CONFIRMATION` — Require confirmation for MCP `post_message` (default `true`).

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
- `cargo check` and `cargo clippy` require `DATABASE_URL` because the server crate
  uses SQLx online query macros (`sqlx::query!`).
- `server/src/main.rs` starts the full Axum HTTP server with WebSocket and MCP
  support when enabled.
- `migrations` is a Cargo workspace member, not just a directory of SQL files.
- Repository traits live in `ruckchat-domain`; SQLx implementations live in
  `server/src/repositories/`.

- The desktop client hard-codes `http://localhost:3000` for development.
  WebSocket authentication relies on the HTTP-only `ruckchat_session` cookie
  set at login; restoring from `localStorage` alone is not sufficient.

## Implementation Order

1. Cargo workspace → 2. Shared crates → 3. Database schema → 4. Domain layer →
5. Services → 6. REST API → 7. WebSocket server → 8. MCP server → 9. Plugin SDK →
10. Desktop → 11. Mobile → 12. Migration tools.

Ship unit tests, integration tests, OpenAPI updates, and docs with every feature.
