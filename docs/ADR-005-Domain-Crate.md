# ADR 005: Domain Layer Crate

## Status

Accepted

## Context

RuckChat needs a place for the core domain model that is independent of HTTP,
database, and client concerns. The server will later implement services and
repositories, while clients and plugins will need to reference the same entity
and event types. Keeping domain logic inside the server crate would couple it
to Axum and SQLx details and make reuse by other workspace members awkward.

## Decision

Introduce a shared `crates/ruckchat-domain` crate that contains:

- Domain entities (`User`, `Organization`, `Channel`, `Message`, etc.).
- Value objects and enums (`Role`, `ConversationType`).
- Repository interfaces (`UserRepository`, `ChannelRepository`, etc.) defined
  with `async-trait` so the server can provide SQLx implementations later.
- Unit tests for domain invariants.

The crate deliberately excludes:

- SQLx queries or connection handling.
- HTTP request/response types.
- WebSocket event serialization.
- Authentication hashing, session signing, or file storage backends.

Domain services that orchestrate repositories and emit side effects will be
implemented in the server crate during the services phase.

## Consequences

- The service layer can depend on repository traits without knowing about SQLx.
- Multiple crates (server, desktop bridge, plugin SDK, MCP server) can share
  the same entity definitions.
- Domain invariants are enforced in plain Rust and can be unit tested without
  a database.
- Repository trait signatures may evolve as service needs become clearer, but
  the aggregate boundaries are stable.
