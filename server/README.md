# ruckchat-server

RuckChat server crate. It implements the Phase 3 service layer and SQLx
repository implementations on top of the `ruckchat-domain` crate. HTTP handlers,
WebSocket, MCP, and plugin support are added in later phases.

## Crate layout

```text
server/src
├── lib.rs               # Crate entry point and database connection helper
├── error.rs             # Server-specific error variants
├── state.rs             # Shared application state
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
│   ├── direct_message.rs
│   └── file.rs
└── testing.rs          # In-memory mock repositories for unit tests
```

## Running tests

Integration tests require a running PostgreSQL database and `DATABASE_URL`:

```bash
export DATABASE_URL="postgres://ruckchat:ruckchat@localhost/ruckchat"
cargo test -p ruckchat-server
```

`connect_database` applies pending migrations from the `ruckchat-migrations`
crate on startup.

## Service layer

Services live in `server/src/services` and depend only on the domain repository
traits defined in `ruckchat-domain`. The current services cover:

- **Auth** — registration, login, logout, session authentication, and session cleanup.
- **Authorization** — role-based and ownership-based permission checks.
- **User** — profile retrieval and updates, and organization member listing.
- **Organization** — create, list, invite, role changes, and member removal.
- **Channel** — create, list, update, archive/unarchive, and membership management.
- **Message** — post, edit, delete, history, and thread replies.
- **DirectMessage** — start conversations and list conversations for a user.
- **File** — record uploads, list files, and attach files to messages.
