# 006 - Server

## Server Crate

The `server` crate is the main Rust application. Phases 1–3 laid the
foundation, and Phase 4 added the Axum REST API. WebSocket, MCP, plugin
loading, and background tasks are added in later phases.

## Technology Stack

| Concern | Library |
|---------|---------|
| HTTP framework | Axum |
| Async runtime | Tokio |
| Database access | SQLx |
| Password hashing | argon2 |
| Serialization | serde + serde_json |
| Configuration | ruckchat-config |
| Logging/tracing | tracing + tracing-subscriber |
| Middleware | tower-http (CORS, trace) |

## Crate Layout

```text
server/src
├── main.rs              # Entry point: config, tracing, DB, graceful shutdown
├── lib.rs               # Crate entry point and database connection helper
├── error.rs             # Server-specific error variants
├── state.rs             # Shared application state and service wiring
├── repositories/        # SQLx data access
│   ├── user.rs
│   ├── session.rs
│   ├── organization.rs
│   ├── organization_membership.rs
│   ├── organization_settings.rs
│   ├── channel.rs
│   ├── channel_membership.rs
│   ├── message.rs
│   ├── direct_message_conversation.rs
│   ├── reaction.rs
│   └── file.rs
├── services/            # Business logic
│   ├── auth.rs
│   ├── authorization.rs
│   ├── user.rs
│   ├── organization.rs
│   ├── channel.rs
│   ├── message.rs
│   ├── direct_message.rs
│   └── file.rs
├── handlers/            # HTTP route handlers and DTOs
│   ├── auth.rs
│   ├── channel.rs
│   ├── direct_message.rs
│   ├── dto.rs
│   ├── error.rs
│   ├── file.rs
│   ├── message.rs
│   ├── mod.rs
│   ├── organization.rs
│   └── user.rs
└── testing.rs           # In-memory mock repositories for unit tests
```

## Service Layer

Services orchestrate domain aggregates and repository traits. They are
independent of HTTP/WebSocket infrastructure and are covered by unit tests with
mock repositories and integration tests against PostgreSQL.

Implemented services:

- **AuthService** — registration, login, logout, session authentication, and
  expired-session cleanup.
- **AuthorizationService** — role-based and ownership-based permission checks.
- **UserService** — profile retrieval/updates and organization member listing.
- **OrganizationService** — create, list, invite, role changes, and member removal.
- **ChannelService** — create, list, update, archive/unarchive, and membership management.
- **MessageService** — post, edit, delete, history, and thread replies.
- **DirectMessageService** — start DM conversations and list conversations for a user.
- **FileService** — record uploads, list files, and attach files to messages.

## Repository Layer

Each domain repository trait has a SQLx implementation in `server/src/repositories`.
Repository implementations use SQLx compile-time checked macros where possible.
The session repository uses runtime queries to work around PostgreSQL `INET`
type inference issues.

## HTTP Layer

`server/src/handlers/mod.rs` wires the Axum router. All endpoints are listed in
`server/openapi.yaml` with full request/response schemas.

### Authentication

The `AuthUser` extractor (`server/src/handlers/auth.rs`) validates the session
token from either:

1. The `ruckchat_session` HTTP-only cookie.
2. The `Authorization: Bearer <token>` header.

On login, the server returns the token in both the JSON body and the cookie.
Logout deletes the session from the database and the client should clear the
cookie locally.

### Error Handling

`ruckchat_common::Error` provides shared variants: `NotFound`, `Unauthorized`,
`Forbidden`, `Validation`, `Conflict`, and `Internal`. `handlers::error::ErrorBody`
maps every variant to a uniform JSON response and the appropriate HTTP status
(400, 401, 403, 404, 409, or 500). JSON extraction failures return 422.

## Request Lifecycle

1. Axum receives a request.
2. `TraceLayer` and `CorsLayer` process it.
3. The `AuthUser` extractor validates the session cookie or bearer token.
4. The matched handler deserializes and validates the request body.
5. The handler calls a service function with the application state and user context.
6. The service enforces domain invariants and calls repositories.
7. The repository executes SQLx queries within the pool.
8. The handler returns a typed response or mapped error.

## Configuration

Server configuration is loaded from `ruckchat.toml` and environment variables. The
required variables for the current phase are:

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |

Additional configuration variables are documented in the configuration crate.

## Startup Sequence

1. Load configuration.
2. Initialize tracing.
3. Connect to PostgreSQL and run pending migrations via `connect_database`.
4. Build service dependencies backed by SQLx repositories in `AppState::from_pool`.
5. Bind the HTTP server to the address derived from `base_url`.
6. Wait for a shutdown signal and drain open requests.

## Testing

Unit tests exercise the service layer against in-memory mocks in
`server/src/testing.rs`. Integration tests under `server/tests/` use `sqlx::test`
to get a fresh PostgreSQL database per test and the `TestClient` helper to
exercise the Axum router in-process.

```bash
export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
cargo test -p ruckchat-server
```

## Shutdown

The server waits for `ctrl+c` and drains open requests before exiting. WebSocket
close events, plugin shutdown hooks, and background tasks are implemented in
later phases.
