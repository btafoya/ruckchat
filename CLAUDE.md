
# CLAUDE IMPLEMENTATION CONTRACT

Never change architecture without updating ADRs.

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
- Phase 8 (in progress): Desktop client scaffold in `desktop/` with Tauri v2,
  React 19, TypeScript, Tailwind CSS v4, and React Router v7. The
  `desktop/src-tauri` crate is part of the Cargo workspace. Native integrations,
  messaging UI, and release packaging follow in subsequent tasks.
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
  `docs/ADR-007-MCP-Server.md` — Active ADRs.

## Environment

Required for integration tests and the running server:
- `DATABASE_URL` — PostgreSQL connection string, e.g.
  `postgres://ruckchat:ruckchat@localhost/ruckchat`.

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

## CodeGraph and MCP Tooling

This project uses the [CodeGraph MCP server](https://colbymchenry.github.io/codegraph/getting-started/introduction/)
for structural code exploration. Prefer CodeGraph tools for questions like:

- Where is X defined?
- What calls function Y?
- What would break if I changed Z?
- Show me focused context for a task/area.
- See several related symbols' source at once.

Rules of thumb:

- Use `codegraph_explore` instead of `grep` for symbol lookup and structural questions.
- Trust the AST-parsed results; do not re-verify with grep.
- Avoid chaining many `Read` calls; one `codegraph_explore` call returns grouped source.

MCP server configuration and additional Claude tooling guidance is available at
[SuperClaude MCP docs](https://superclaude.netlify.app/docs/User-Guide/mcp-servers).
Use configured MCP servers whenever they provide a dedicated tool for the task.

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

Before planning, read the relevant project documentation. Priority order:

1. Active ADRs in `docs/ADR-*.md` for architectural constraints.
2. `book/000-Vision.md`, `book/001-Product.md`, and `book/002-UX.md` for goals and behavior expectations.
3. `book/003-Architecture.md`, `book/004-Domain.md`, and `book/005-Database.md` for structure and invariants.
4. `book/006-Server.md` for server-layer conventions.
5. `.claude/plan.md` if the task is part of an active phase plan.

### Plan

- State assumptions explicitly.
- Identify which crates, services, repositories, and domain invariants are involved.
- Decide whether the change requires an ADR update before code changes.
- For non-trivial changes, write the plan down and verify it against the docs before coding.

### Write code

- Follow the phase order in **Implementation Order**.
- Keep changes surgical; do not refactor unrelated code.
- Match existing style and naming.

### Format, check, lint, test

| Step | Command | Stop if it fails |
|------|---------|------------------|
| Format | `cargo fmt --all` | Yes |
| Check | `cargo check --workspace` | Yes |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Yes |
| Test | `cargo nextest run --workspace` | Yes |

If `cargo nextest` is not installed, use `cargo test --workspace` as a fallback.

### Update docs

After the code passes all checks, update the relevant documentation:

- `server/openapi.yaml` for REST API changes.
- `book/*.md` for behavior, architecture, or convention changes.
- `docs/ADR-*.md` for architecture decisions or changes.
- `server/README.md` for crate-specific developer guidance.
- This `CLAUDE.md` for global contract changes.

### Commit

- Author commits as `Brian Tafoya <btafoya@briantafoya.com>`.
- Do not include AI attribution in commit messages or code comments.
- Never commit `.env` files or secrets.

### Update codegraph

After committing, refresh the CodeGraph index so future structural queries reflect the new code:

```bash
codegraph refresh
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

## Implementation Order

1. Cargo workspace
2. Shared crates
3. Database schema
4. Domain layer
5. Services
6. REST API
7. WebSocket server
8. MCP server
9. Plugin SDK
10. Desktop
11. Mobile
12. Migration tools

Every completed feature must include:
- Unit tests
- Integration tests
- OpenAPI updates
- Documentation
