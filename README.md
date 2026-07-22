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

Phases 1–4 are complete. The server is a working REST API with authentication,
organizations, channels, direct messages, file metadata, and integration tests
against PostgreSQL.

| Phase | Status | Description |
|-------|--------|-------------|
| 1 | ✅ Complete | Cargo workspace, shared crates, database migrations |
| 2 | ✅ Complete | Domain layer: entities, value objects, repository traits |
| 3 | ✅ Complete | Service layer and SQLx repository implementations |
| 4 | ✅ Complete | Axum REST API, auth extractor, integration tests |
| 5 | ✅ Complete | WebSocket server for realtime messaging |
| 6 | Planned | MCP server integration |
| 7 | Planned | Plugin SDK |
| 8 | Planned | Desktop client (Tauri) |
| 9 | Planned | Mobile client (Flutter) |
| 10 | Planned | Migration and packaging tools |

## Tech Stack

- **Server**: Rust, Axum, Tokio, SQLx
- **Database**: PostgreSQL 16+
- **Authentication**: Argon2 password hashing, SHA-256 session tokens
- **Validation**: Custom domain validators in `ruckchat-common`
- **Configuration**: `ruckchat-config` (TOML + environment variables)
- **Tracing**: `tracing` + `tracing-subscriber`

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.94 or later
- [PostgreSQL](https://www.postgresql.org/download/) 16 or later
- A running PostgreSQL database and `DATABASE_URL`

### Run the Server

```bash
export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
cargo sqlx migrate run --source migrations/migrations
cargo run -p ruckchat-server
```

The server binds to `http://localhost:3000` by default.

### Run the Tests

```bash
# Unit tests (no database required)
cargo test --workspace

# Server integration tests (requires PostgreSQL)
export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
cargo test -p ruckchat-server
```

## API

The REST API is documented in [server/openapi.yaml](server/openapi.yaml).

Authentication accepts either an HTTP-only `ruckchat_session` cookie or an
`Authorization: Bearer <token>` header.

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
| POST | `/direct_messages` | Start a DM conversation |

## Architecture

```text
root/
├── crates/
│   ├── ruckchat-id/        # Strongly-typed UUID wrappers
│   ├── ruckchat-common/    # Shared errors and validation
│   ├── ruckchat-config/    # Configuration primitives
│   └── ruckchat-domain/    # Entities and repository traits
├── server/                 # HTTP handlers, services, SQLx repositories
│   ├── src/handlers/       # Axum routes
│   ├── src/services/       # Business logic
│   ├── src/repositories/   # SQLx implementations
│   └── tests/              # HTTP integration tests
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
- [server/README.md](server/README.md) — Crate-specific developer guide
- [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) — Sprint plan
- [docs/ADR-*.md](docs/) — Architecture Decision Records

## Roadmap

See [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for the full
sprint breakdown. Upcoming milestones include WebSocket realtime messaging,
a Tauri desktop client, a Flutter mobile client, and a plugin SDK.

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
