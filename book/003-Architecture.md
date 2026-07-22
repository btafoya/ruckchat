# 003 - Architecture

## System Shape

RuckChat is a monolithic application with a clear client-server boundary.

- **Server:** A single Rust process built on Axum and Tokio.
- **Database:** A single PostgreSQL database accessed through SQLx.
- **Desktop Client:** Tauri + React.
- **Mobile Client:** Flutter.
- **Real-time Transport:** WebSockets for message delivery and presence.
- **File Storage:** Local filesystem by default; S3-compatible object storage is optional.

## Server Layers

```
HTTP / WebSocket (Axum)
        |
  Handler / Router layer
        |
  Service layer (domain logic)
        |
  Repository layer (SQLx queries)
        |
  PostgreSQL
```

### Handler Layer

- Axum routers and extractors.
- Input validation using strongly-typed request structs.
- Error conversion into HTTP responses.
- OpenAPI annotations maintained alongside route definitions.

### Service Layer

- Encapsulates business logic and domain invariants.
- Orchestrates repositories and side effects (notifications, file storage, WebSocket events).
- Returns domain-specific errors that the handler layer maps to HTTP status codes.

### Repository Layer

- Direct SQLx queries against PostgreSQL.
- One repository per aggregate (users, organizations, channels, messages, files).
- Queries are written as plain SQL; compile-time checking via `sqlx::query_as!` macros.

### Shared Crates

The workspace shared crates (`ruckchat-common`, `ruckchat-config`, `ruckchat-id`,
and `ruckchat-domain`) contain:

- Common error types and validation utilities.
- Strongly-typed identifiers and configuration primitives.
- Domain entities and repository traits.

WebSocket event types and the event bus trait live in the `ruckchat-server`
crate so that transport concerns do not leak into shared code.

## Client Architecture

### Desktop (Tauri + React)

- React front end runs in a WebView.
- Tauri Rust bridge exposes native APIs for notifications, file system access, and deep links.
- State management uses React hooks and context; no mandatory global state library.
- Communicates with the server via REST and WebSocket.

### Mobile (Flutter)

- Single Flutter codebase for Android and iOS.
- State management uses Riverpod or Provider (chosen once and applied consistently).
- REST and WebSocket communication via Dart `http` and `web_socket_channel` packages.
- Platform-specific push notifications handled through Flutter local notifications in v1; Firebase/APNs only if required later.

## Real-Time Events

- WebSocket connections are authenticated using the same session cookie or bearer
  token as REST requests.
- `server/src/websocket/manager.rs` tracks active sockets per user and
  organization in memory.
- `server/src/services/events.rs` defines the `EventBus` trait and event
  envelopes, keeping services decoupled from transport.
- `server/src/websocket/bus.rs` implements the bus, resolves recipients using
  repository data, and dispatches through the connection manager.
- Events are broadcast to relevant recipients:
  - `message.created`
  - `message.updated`
  - `message.deleted`
  - `reaction.added`
  - `reaction.removed`
  - `presence.updated`
  - `typing.updated`

## Plugin Architecture

- Plugins run in-process as native Rust dynamic libraries loaded at startup.
- The server exposes a stable ABI and SDK for hooks.
- Plugins cannot directly access the database; they interact through the plugin API.

## Deployment Shape

```
+-----------+     +-----------+
|  Desktop  |     |  Mobile   |
|  Client   |     |  Client   |
+-----+-----+     +-----+-----+
      |                 |
      +---------+-------+
                |
           +----v----+   +------------+
           |  Caddy  |-->|   RuckChat  |
           |  Proxy  |   |   Server    |
           +----+----+   +------+------+
                |               |
                |          +----v------+
                |          | PostgreSQL|
                |          |  Database |
                |          +-----------+
           +----v----+
           |  Static  |
           |  Files   |
           +----------+
```

## Excluded Components

The v1 architecture intentionally omits:

- Redis, Kafka, RabbitMQ, or other message brokers.
- Elasticsearch or other dedicated search servers.
- Kubernetes or service mesh.
- Microservices or separate real-time services.

All shared state lives in PostgreSQL or in-memory within the single server process.
