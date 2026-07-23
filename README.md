# RuckChat

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.94+-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16+-4169E1?logo=postgresql&logoColor=white)](https://www.postgresql.org)
[![Axum](https://img.shields.io/badge/Axum-0.8-000000?logo=rust&logoColor=white)](https://github.com/tokio-rs/axum)
[![GitHub Repo](https://img.shields.io/badge/GitHub-btafoya%2Fruckchat-181717?logo=github&logoColor=white)](https://github.com/btafoya/ruckchat)

A lean, open-source team chat platform built with Rust, PostgreSQL, and Axum.

RuckChat is designed for small teams that want a self-hosted Slack alternative
without the operational overhead of microservices, Redis, Kafka, Elasticsearch,
or Kubernetes. One Rust server, one PostgreSQL database, and a handful of
clients.

## Current Status

Phases 1–9 are complete. The server is a working REST API with authentication,
organizations, channels, direct messages, file metadata, WebSocket real-time
events, an MCP server, a native plugin SDK, and runtime YAML configuration, with
integration tests against PostgreSQL. The desktop client provides a Tauri + React
UI with messaging, native OS notifications, a system tray icon with unread count,
`ruckchat://` deep links, configurable backend URL, draft persistence, and
failed-send retry.

| Phase | Status | Description |
|-------|--------|-------------|
| 1 | ✅ Complete | Cargo workspace, shared crates, database migrations |
| 2 | ✅ Complete | Domain layer: entities, value objects, repository traits |
| 3 | ✅ Complete | Service layer and SQLx repository implementations |
| 4 | ✅ Complete | Axum REST API, auth extractor, integration tests |
| 5 | ✅ Complete | WebSocket server for realtime messaging |
| 6 | ✅ Complete | MCP server integration |
| 7 | ✅ Complete | Plugin SDK and native dynamic plugins |
| 8 | ✅ Complete | Desktop client (Tauri + React) |
| 9 | ✅ Complete | Runtime YAML configuration |
| 10 | Planned | Mobile client (Flutter) |
| 11 | Planned | Migration and packaging tools |

## Tech Stack

- **Server**: Rust, Axum, Tokio, SQLx
- **Database**: PostgreSQL 16+
- **Authentication**: Argon2 password hashing, SHA-256 session tokens
- **Validation**: Custom domain validators in `ruckchat-common`
- **Configuration**: Runtime YAML (`ruckchat.yaml`) via `ruckchat-config`
- **Real-time**: WebSocket with in-memory connection manager
- **MCP**: `rmcp` Streamable HTTP transport at `/mcp/v1/sse`
- **Plugins**: Native Rust dynamic libraries loaded at startup
- **Tracing**: `tracing` + `tracing-subscriber`
- **Desktop**: Tauri v2, React 19, TypeScript, Vite, Tailwind CSS v4

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.94 or later
- [PostgreSQL](https://www.postgresql.org/download/) 16 or later
- A running PostgreSQL database

### Run the Server

```bash
set -a
source .env.testing
set +a

cargo run -p ruckchat-server -- --init-config ./ruckchat.yaml
# edit ./ruckchat.yaml, then:
cargo run -p ruckchat-server -- --config ./ruckchat.yaml
```

The server binds to the address derived from `base_url` in `ruckchat.yaml`
(`http://localhost:3000` by default).

### Run the Desktop Client

```bash
cd desktop
pnpm install
pnpm tauri dev
```

The desktop client opens a Tauri WebView pointing at the Vite dev server. It
expects the server at `http://localhost:3000` by default; change this from the
`/settings` screen in the app.

### Run the Tests

```bash
# All workspace tests (server integration tests require DATABASE_URL)
set -a
source .env.testing
set +a
cargo nextest run --workspace

# Server crate only
cargo test -p ruckchat-server
```

If you do not have `cargo nextest`, use `cargo test --workspace`.

Schema/migration tests that create isolated per-test databases read
`RUCKCHAT_TEST_ADMIN_DATABASE_URL` (default:
`postgres://postgres:postgres@localhost:5445/postgres`).

## API

The REST API is documented in [server/openapi.yaml](server/openapi.yaml).

Authentication accepts either an HTTP-only `ruckchat_session` cookie or an
`Authorization: Bearer <token>` header.

Real-time updates are delivered over the authenticated WebSocket at
`/websocket`.

Key endpoints:

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/register` | Create a user and initial organization |
| POST | `/auth/login` | Start a session |
| POST | `/auth/logout` | End the current session |
| GET  | `/users/me` | Get the authenticated profile |
| GET  | `/organizations` | List my organizations |
| POST | `/organizations` | Create an organization |
| GET  | `/organizations/{id}/channels` | List channels |
| POST | `/organizations/{id}/channels` | Create a channel |
| GET  | `/channels/{id}/messages` | Get channel history |
| POST | `/channels/{id}/messages` | Post a message |
| GET  | `/websocket` | Upgrade to authenticated WebSocket |
| POST | `/plugins/{plugin}/commands/{command}` | Invoke a plugin slash command |
| POST | `/mcp/v1/sse` | MCP Streamable HTTP client messages |
| GET  | `/mcp/v1/sse` | MCP Server-Sent Events stream |

## Architecture

```text
root/
├── crates/
│   ├── ruckchat-id/        # Strongly-typed UUID wrappers
│   ├── ruckchat-common/    # Shared errors and validation
│   ├── ruckchat-config/    # Configuration primitives and runtime YAML parsing
│   ├── ruckchat-domain/    # Entities and repository traits
│   └── ruckchat-plugin-sdk/# Plugin SDK trait, types, and `declare_plugin!` macro
├── server/                 # HTTP handlers, services, SQLx repositories
│   ├── src/handlers/       # Axum routes
│   ├── src/services/       # Business logic
│   ├── src/repositories/   # SQLx implementations
│   ├── src/websocket/      # WebSocket real-time events
│   ├── src/mcp/            # MCP server
│   ├── src/plugins/        # Dynamic plugin loader, manager, host API, event bus
│   └── tests/              # HTTP integration tests
├── desktop/                # Tauri v2 + React desktop client
│   ├── src/                # React + TypeScript frontend
│   ├── src-tauri/          # Tauri Rust shell
│   └── README.md           # Desktop developer guide
├── migrations/             # SQLx migrations
├── book/                   # Project documentation
└── docs/                   # ADRs and implementation plans
```

Repository traits live in `ruckchat-domain`; SQLx implementations live in
`server/src/repositories`. Services are unit-tested against in-memory mocks in
`server/src/testing.rs` and integration-tested against PostgreSQL in
`server/tests/`.

## Documentation

- [book/006-Server.md](book/006-Server.md) — Server conventions and request lifecycle
- [book/007-Desktop.md](book/007-Desktop.md) — Desktop client conventions and build notes
- [server/README.md](server/README.md) — Crate-specific developer guide
- [desktop/README.md](desktop/README.md) — Desktop client developer guide
- [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) — Sprint plan
- [docs/ADR-*.md](docs/) — Architecture Decision Records

## Roadmap

See [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for the full
sprint breakdown. Upcoming milestones include a Flutter mobile client and
migration/packaging tools.

## Contributing

Contributions are welcome. Please open an issue or pull request on
[GitHub](https://github.com/btafoya/ruckchat). All code must pass:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace
```

See [CLAUDE.md](CLAUDE.md) for the project's development contract and
implementation loop.

## License

RuckChat is licensed under the [MIT License](LICENSE).

## Author

Brian Tafoya — [btafoya@briantafoya.com](mailto:btafoya@briantafoya.com)
