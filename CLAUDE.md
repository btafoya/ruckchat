
# CLAUDE IMPLEMENTATION CONTRACT

Never change architecture without updating ADRs.

## Current Status

Phases 1–3 are complete. Phases 4–12 are not yet implemented.

- Phase 1: Cargo workspace, shared crates (`ruckchat-id`, `ruckchat-common`,
  `ruckchat-config`), database migrations, and schema integration tests.
- Phase 2: Domain layer (`ruckchat-domain` crate) with entities, value objects,
  and repository traits.
- Phase 3: Service layer and SQLx repositories in `ruckchat-server`, plus
  unit-tested service logic using in-memory mocks.
- `server/src/main.rs` is currently a placeholder binary. The full REST API,
  WebSocket, and MCP server will be added in later phases.

## Commands

| Command | Description |
|---------|-------------|
| `cargo build --workspace` | Build all crates |
| `cargo test --workspace` | Run unit tests across all crates |
| `cargo test -p ruckchat-server` | Run server tests (requires `DATABASE_URL` for integration tests) |
| `cargo clippy --workspace -- -D warnings` | Run clippy with workspace lints |
| `cargo sqlx migrate run --source migrations/migrations` | Apply pending migrations |
| `cargo run -p ruckchat-server` | Run the placeholder server binary |

## Architecture

```text
root/
├── crates/
│   ├── ruckchat-id/        # Strongly-typed IDs
│   ├── ruckchat-common/    # Shared error type and validation utilities
│   ├── ruckchat-config/    # Configuration primitives and `AuthenticatedUser`
│   └── ruckchat-domain/    # Entities, value objects, and repository traits
├── server/                 # Service layer and SQLx repository implementations
├── migrations/             # SQLx migration crate and SQL files
├── book/                   # mdBook-style project documentation
├── docs/
│   └── ADR-*.md            # Architecture Decision Records
└── server/openapi.yaml     # REST API stub for Phase 3 use cases
```

## Key Files

- `Cargo.toml` — Workspace manifest with shared dependencies and strict lints.
- `server/src/lib.rs` — Server crate entry point and `connect_database` helper.
- `server/src/services/` — Business logic and DTOs.
- `server/src/repositories/` — SQLx implementations of domain repository traits.
- `server/src/testing.rs` — In-memory mock repositories for service unit tests.
- `migrations/migrations/` — SQLx `.up.sql` / `.down.sql` migration files.
- `server/openapi.yaml` — Stub OpenAPI document for implemented use cases.
- `docs/ADR-003-Shared-Crates.md`, `docs/ADR-004-Migrations.md`,
  `docs/ADR-005-Domain-Crate.md` — Active ADRs.

## Environment

Required for integration tests and the running server:
- `DATABASE_URL` — PostgreSQL connection string, e.g.
  `postgres://ruckchat:ruckchat@localhost/ruckchat`.

Optional via `ruckchat.toml` or `RUCKCHAT_*` environment variables:
- `RUCKCHAT_APP_NAME`
- `RUCKCHAT_ENVIRONMENT`
- `RUCKCHAT_BASE_URL`
- `RUCKCHAT_LOG_LEVEL`

## Testing

- `cargo test --workspace` runs unit tests without a database.
- `ruckchat-server` integration tests require a running PostgreSQL database and
  `DATABASE_URL`. `connect_database` applies pending migrations on startup.
- Services are unit-tested against in-memory mocks in `server/src/testing.rs`,
  not against the real database.

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
Update codegraph
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
- The binary in `server/src/main.rs` is a stub; the full HTTP/WebSocket/MCP
  server implementation is intentionally deferred.
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
