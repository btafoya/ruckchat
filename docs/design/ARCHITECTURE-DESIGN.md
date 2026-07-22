# RuckChat v1 Architecture Design

## 1. Design Goals

This document defines the system architecture for RuckChat v1. It is driven by the requirements in `docs/requirements/RUCKCHAT-REQUIREMENTS.md` and the constraints in `book/003-Architecture.md`.

### Constraints

- Single Rust server process.
- PostgreSQL as the only database.
- No Redis, Kafka, Elasticsearch, Kubernetes, or microservices.
- Tauri + React desktop client.
- Flutter mobile client.
- REST + WebSocket from the same server.

## 2. High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                         Clients                               │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐  │
│  │   Desktop   │      │    Mobile   │      │  MCP Client │  │
│  │ Tauri/React │      │   Flutter   │      │  (SSE/HTTP) │  │
│  └──────┬──────┘      └──────┬──────┘      └──────┬──────┘  │
└─────────┼─────────────────────┼─────────────────────┼────────┘
          │                     │                     │
          └─────────────────────┬─────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │  Caddy Reverse Proxy  │
                    │   TLS termination     │
                    └───────────┬───────────┘
                                │
                    ┌───────────▼───────────┐
                    │    RuckChat Server    │
                    │      (Axum/Tokio)     │
                    │                       │
                    │  ┌─────┐  ┌────────┐  │
                    │  │REST │  │WebSocket│ │
                    │  │ API │  │ Manager│  │
                    │  └──┬──┘  └───┬────┘  │
                    │     │         │       │
                    │  ┌──▼─────────▼──┐   │
                    │  │  Service Layer  │   │
                    │  │  (domain logic) │   │
                    │  └────────┬────────┘   │
                    │           │            │
                    │  ┌────────▼────────┐   │
                    │  │ Repository Layer │   │
                    │  │     (SQLx)       │   │
                    │  └────────┬────────┘   │
                    └───────────┼────────────┘
                                │
                    ┌───────────▼───────────┐
                    │       PostgreSQL       │
                    │  (data + search state) │
                    └─────────────────────────┘
```

## 3. Server Component Design

### 3.1 Layers

| Layer | Responsibility | Crate/Module |
|-------|----------------|--------------|
| Router | Route dispatch, middleware, auth extraction | `server/src/router.rs` |
| Handlers | Request validation, response construction | `server/src/handlers/` |
| Services | Business rules, authorization, event orchestration | `server/src/services/` |
| Repositories | SQL queries, transaction boundaries | `server/src/repositories/` |
| WebSocket Manager | Connection registry, event fan-out | `server/src/websocket/` |
| Plugin Host | Plugin lifecycle and API | `server/src/plugins/` |

### 3.2 Shared Crate

The `shared` crate is the contract between server and clients:

- DTOs for REST request/response bodies.
- WebSocket event types.
- Domain error codes.
- Validation constants (max message length, allowed channel name characters, etc.).

### 3.3 Application State

The Axum application state holds:

- `PgPool` from SQLx.
- `WebSocketManager` handle.
- `FileStorage` backend.
- `PluginHost` handle.
- Configuration object.
- Background task cancellation token.

### 3.4 Request Flow

```
Client Request
    │
    ▼
Caddy (TLS)
    │
    ▼
Axum Router
    │
    ▼
Auth Middleware (session cookie → user_id)
    │
    ▼
Handler (validate input DTO)
    │
    ▼
Service (enforce rules, orchestrate)
    │
    ▼
Repository (execute SQLx query)
    │
    ▼
PostgreSQL
    │
    ▼
Service emits side effects (WS event, email job)
    │
    ▼
Handler returns DTO / error
```

### 3.5 WebSocket Manager

The WebSocket manager is in-memory and single-process:

- `connections: HashMap<UserId, Vec<ConnectionHandle>>`
- `organization_subscribers: HashMap<OrganizationId, HashSet<UserId>>`
- Events are routed by calculating the target audience from the conversation and organization.
- Typing and presence events are debounced before broadcast.

### 3.6 Background Tasks

| Task | Frequency | Purpose |
|------|-----------|---------|
| Email notifier | Every 60 seconds | Send queued email notifications |
| File janitor | Every 24 hours | Remove orphaned file metadata and storage |
| Presence sweeper | Every 30 seconds | Mark disconnected users offline |

## 4. Client Architecture

### 4.1 Desktop (Tauri + React)

```
┌─────────────────────────────┐
│       React UI Layer        │
│  - Components (Tailwind)    │
│  - State (hooks + context)  │
│  - Routing                  │
└─────────────┬───────────────┘
              │
┌─────────────▼───────────────┐
│      API Client             │
│  - REST fetch               │
│  - WebSocket wrapper        │
└─────────────┬───────────────┘
              │
┌─────────────▼───────────────┐
│      Tauri Bridge           │
│  - Notifications            │
│  - File dialogs             │
│  - Deep links               │
└─────────────────────────────┘
```

### 4.2 Mobile (Flutter)

```
┌─────────────────────────────┐
│      Flutter UI Layer       │
│  - Screens / Widgets        │
│  - State (Riverpod)         │
└─────────────┬───────────────┘
              │
┌─────────────▼───────────────┐
│      Data Layer             │
│  - API service              │
│  - WebSocket service        │
│  - Local cache              │
└─────────────────────────────┘
```

## 5. Data Flow Examples

### 5.1 Sending a Message

1. Client sends `POST /api/v1/conversations/:id/messages` with content.
2. Handler validates the request.
3. `MessageService` checks membership and rate limits.
4. `MessageRepository` inserts the message in a transaction.
5. `MessageService` emits `message.created` to the WebSocket manager.
6. WebSocket manager fans out to users in the conversation.
7. Handler returns the created message DTO.

### 5.2 Real-Time Reaction

1. Client sends `POST /api/v1/messages/:id/reactions` with emoji.
2. Service validates and updates the `reactions` table.
3. Service emits `reaction.updated` event.
4. WebSocket manager broadcasts to conversation members.
5. Clients update local reaction counts.

### 5.3 File Upload

1. Client sends `multipart/form-data` to `POST /api/v1/files`.
2. Handler streams the file to a temporary location.
3. `FileService` validates MIME type and size.
4. `FileStorage` backend writes the file.
5. `FileRepository` inserts metadata.
6. Handler returns file metadata DTO.

## 6. Deployment Architecture

### 6.1 Minimal Production Deployment

```
┌─────────────────────────────────────┐
│           Internet                  │
└───────────────┬─────────────────────┘
                │
        ┌───────▼────────┐
        │     Caddy      │
        │  TLS / static  │
        └───────┬────────┘
                │
        ┌───────▼────────┐
        │  RuckChat      │
        │  Server        │
        └───────┬────────┘
                │
        ┌───────▼────────┐
        │  PostgreSQL    │
        └────────────────┘
```

### 6.2 With Object Storage

```
        ┌───────▼────────┐
        │  RuckChat      │
        │  Server        │
        └───────┬────────┘
                │
    ┌───────────┴───────────┐
    ▼                       ▼
┌─────────┐            ┌─────────┐
│PostgreSQL│            │   S3    │
└─────────┘            └─────────┘
```

## 7. Failure Handling

| Failure | Mitigation |
|---------|------------|
| Database connection loss | Return 500, retry pool, alert operator |
| WebSocket disconnect | Client reconnects, fetch missed history via REST |
| Plugin crash | Isolate plugin, log error, continue server |
| Slow WebSocket client | Drop non-critical events, eventually close |
| File storage backend unavailable | Return upload error, queue for retry |

## 8. Design Decisions

### 8.1 Why a Single Process

The v1 requirement is simplicity. A single process avoids distributed coordination, service discovery, and inter-service networking. Scaling is deferred to a future version.

### 8.2 Why PostgreSQL Full-Text Search

It satisfies the search requirement without adding Elasticsearch or another service. Performance is acceptable for the target message volume.

### 8.3 Why Native Rust Plugins

Native plugins provide performance and type safety. The tradeoff is trust: operators must only install plugins they trust. Sandboxing is deferred.

### 8.4 Why No Background Mobile Push in v1

Firebase Cloud Messaging and Apple Push Notification service introduce external dependencies and platform-specific credentials. Foreground notifications and resume reconciliation meet the immediate need.

## 9. Interfaces

### 9.1 Service-to-Repository

Services call repositories with strongly-typed inputs and receive domain models or `Result` types. Repositories do not enforce authorization; services do.

### 9.2 Service-to-WebSocket Manager

Services publish events by calling `WebSocketManager::broadcast(event, audience)`. The manager resolves the audience to active connections.

### 9.3 Server-to-Clients

- REST over HTTP/1.1 or HTTP/2.
- WebSocket over HTTP upgrade.
- MCP over Server-Sent Events.

## 10. Risks and Tradeoffs

| Risk | Mitigation |
|------|------------|
| Single process is a single point of failure | Documented; vertical scaling and backups are the v1 answer |
| In-memory WebSocket state limits horizontal scaling | Out of scope for v1; revisit in v2 |
| Native plugins can crash the server | Load only trusted plugins; isolate in post-MVP |
| PostgreSQL full-text search may not scale to millions of messages | Monitor query times; add trigram indexing if needed |

## 11. Files Produced

- `docs/design/ARCHITECTURE-DESIGN.md` (this file)
- `docs/design/DATABASE-SCHEMA-DESIGN.md`
- `docs/design/OPENAPI-DESIGN.md` (schema and endpoint outline)
