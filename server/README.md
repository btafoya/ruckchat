# ruckchat-server

RuckChat server crate. It implements the service layer, SQLx repository
implementations, the Axum REST API, and the WebSocket real-time event layer on
top of the `ruckchat-domain` crate. MCP and plugin support are added in later
phases.

## Crate layout

```text
server/src
├── lib.rs               # Crate entry point and database connection helper
├── error.rs             # Server-specific error variants
├── state.rs             # Shared application state and service wiring
├── main.rs              # Entry point: config, tracing, DB, graceful shutdown
├── repositories/        # SQLx implementations of domain repository traits
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
├── services/            # Business logic and DTOs
│   ├── auth.rs
│   ├── authorization.rs
│   ├── user.rs
│   ├── organization.rs
│   ├── channel.rs
│   ├── message.rs
│   ├── reaction.rs
│   ├── direct_message.rs
│   ├── file.rs
│   └── events.rs        # EventBus trait and WebSocket event types
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
│   ├── reaction.rs
│   └── user.rs
├── websocket/           # WebSocket connection registry and event bus
│   ├── mod.rs
│   ├── manager.rs
│   ├── bus.rs
│   └── handler.rs
└── testing.rs          # In-memory mock repositories and event bus for unit tests
```

## Running tests

Integration tests require a running PostgreSQL database and `DATABASE_URL`:

```bash
export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
cargo test -p ruckchat-server
```

Unit tests run against in-memory mocks and do not require a database:

```bash
cargo test --workspace
```

`connect_database` applies pending migrations from the `ruckchat-migrations`
crate on startup.

## Running the server

```bash
export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
cargo run -p ruckchat-server
```

The server binds to the address derived from `base_url` in the configuration
(default `http://localhost:3000`) and runs pending migrations before accepting
requests.

## API documentation

The REST API and WebSocket upgrade endpoint are documented in
`server/openapi.yaml`. The WebSocket protocol is documented in
`docs/007-WebSocket-Protocol.md` and `book/010-WebSockets.md`.

## Service layer

Services live in `server/src/services` and depend only on the domain repository
traits defined in `ruckchat-domain`. The current services cover:

- **Auth** — registration, login, logout, session authentication, and session cleanup.
- **Authorization** — role-based and ownership-based permission checks.
- **User** — profile retrieval and updates, and organization member listing.
- **Organization** — create, list, invite, role changes, and member removal.
- **Channel** — create, list, update, archive/unarchive, and membership management.
- **Message** — post, edit, delete, history, and thread replies; emits real-time events.
- **Reaction** — add and remove message reactions; emits real-time events.
- **DirectMessage** — start conversations and list conversations for a user.
- **File** — record uploads, list files, and attach files to messages.

Real-time delivery is implemented in `server/src/websocket`:

- **ConnectionManager** — in-memory registry of active sockets.
- **WebSocketEventBus** — implements the `EventBus` trait, resolves recipients,
  and dispatches events.
- **websocket_handler** — Axum WebSocket upgrade handler.

## HTTP layer

`server/src/handlers/mod.rs` builds the Axum router. Authentication is handled
by the `AuthUser` extractor, which accepts either an HTTP-only `ruckchat_session`
cookie or an `Authorization: Bearer <token>` header. Errors are mapped to a
uniform JSON body by `server/src/handlers/error.rs`.
