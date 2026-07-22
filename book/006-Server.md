# 006 - Server

## Server Crate

The `server` crate is the main Rust application. Phase 3 implemented the service
layer and SQLx-backed repository implementations. HTTP handlers, WebSocket,
MCP, plugin loading, and background tasks are added in later phases.

## Technology Stack

| Concern | Library |
|---------|---------|
| HTTP framework | Axum (future phase) |
| Async runtime | Tokio |
| Database access | SQLx |
| Password hashing | argon2 |
| Serialization | serde + serde_json |
| Configuration | ruckchat-config |
| Logging/tracing | tracing + tracing-subscriber |

## Crate Layout

```text
server/src
├── main.rs              # Entry point stub; full startup in later phases
├── lib.rs               # Crate entry point and database connection helper
├── error.rs             # Server-specific error variants
├── state.rs             # Shared application state
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
├── testing.rs           # In-memory mock repositories for unit tests
└── handlers/            # HTTP route handlers (future phase)
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

## Request Lifecycle (future)

1. Axum receives a request.
2. Middleware extracts and validates the session cookie.
3. The matched handler deserializes and validates the request body.
4. The handler calls a service function with the application state and user context.
5. The service enforces domain invariants and calls repositories.
6. The repository executes SQLx queries within the pool.
7. The service emits side effects (WebSocket events, email jobs, plugin hooks).
8. The handler returns a typed response or mapped error.

## Error Handling

- `ruckchat_common::Error` provides shared variants: `NotFound`, `Unauthorized`,
  `Forbidden`, `Validation`, `Conflict`, and `Internal`.
- The server crate wraps these in `Error::Domain` and adds its own variants for
  password hashing and token generation failures.
- Each service maps SQLx errors to the appropriate domain error.

## Configuration

Server configuration is loaded from environment variables. The required
variables for the current phase are:

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |

Additional configuration variables for later phases are documented in the
configuration crate.

## Startup Sequence (current)

1. Load configuration.
2. Initialize tracing.
3. Connect to PostgreSQL and run pending migrations via `connect_database`.
4. Build service dependencies backed by SQLx repositories.

## Shutdown

Graceful shutdown, open-request draining, WebSocket close events, and plugin
shutdown hooks are implemented in later phases.
