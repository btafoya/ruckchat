# 006 - Server

## Server Crate

The `server` crate is the main Rust application. It exposes HTTP and WebSocket endpoints, runs background tasks, and loads plugins.

## Technology Stack

| Concern | Library |
|---------|---------|
| HTTP framework | Axum |
| Async runtime | Tokio |
| Database access | SQLx |
| Password hashing | argon2 |
| Serialization | serde + serde_json |
| Configuration | figment or envy |
| Validation | validator |
| Logging/tracing | tracing + tracing-subscriber |

## Crate Layout

```
server/src
├── main.rs              # Entry point, configuration, startup
├── config.rs            # Configuration structs and env mapping
├── error.rs             # Application error type and HTTP mapping
├── state.rs             # Shared application state
├── router.rs            # Axum router composition
├── handlers/            # HTTP route handlers
│   ├── auth.rs
│   ├── users.rs
│   ├── organizations.rs
│   ├── channels.rs
│   ├── messages.rs
│   ├── files.rs
│   └── search.rs
├── services/            # Business logic
│   ├── auth_service.rs
│   ├── organization_service.rs
│   ├── channel_service.rs
│   ├── message_service.rs
│   ├── file_service.rs
│   └── notification_service.rs
├── repositories/        # SQLx data access
│   ├── user_repository.rs
│   ├── organization_repository.rs
│   ├── channel_repository.rs
│   ├── message_repository.rs
│   └── file_repository.rs
├── websocket/           # WebSocket manager and event routing
│   ├── manager.rs
│   ├── connection.rs
│   └── events.rs
├── plugins/             # Plugin loader and SDK bindings
│   ├── loader.rs
│   └── host.rs
└── tasks/               # Background tasks
    └── email_notifications.rs
```

## Configuration

Server configuration is loaded from environment variables with sensible defaults:

| Variable | Default | Description |
|----------|---------|-------------|
| `RUCKCHAT_HOST` | `0.0.0.0` | Bind address |
| `RUCKCHAT_PORT` | `3000` | HTTP port |
| `DATABASE_URL` | — | PostgreSQL connection string |
| `DATABASE_MAX_CONNECTIONS` | `10` | Connection pool size |
| `SESSION_SECRET` | — | Secret for cookie signing |
| `SESSION_MAX_AGE_DAYS` | `30` | Session cookie lifetime |
| `PASSWORD_MIN_LENGTH` | `10` | Minimum password length |
| `FILE_STORAGE_BACKEND` | `filesystem` | `filesystem` or `s3` |
| `FILE_STORAGE_PATH` | `./uploads` | Local storage path |
| `S3_ENDPOINT` | — | S3-compatible endpoint |
| `S3_BUCKET` | — | S3 bucket name |
| `S3_ACCESS_KEY` | — | S3 access key |
| `S3_SECRET_KEY` | — | S3 secret key |
| `SMTP_HOST` | — | SMTP server for email notifications |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_FROM` | — | From address for emails |
| `PLUGIN_DIR` | `./plugins` | Directory to scan for plugins |

## Request Lifecycle

1. Axum receives a request.
2. Middleware extracts and validates the session cookie.
3. The matched handler deserializes and validates the request body.
4. The handler calls a service function with the application state and user context.
5. The service enforces domain invariants and calls repositories.
6. The repository executes SQLx queries within the pool.
7. The service emits side effects (WebSocket events, email jobs, plugin hooks).
8. The handler returns a typed response or mapped error.

## Error Handling

- Application errors are represented by a single `AppError` enum.
- Common variants: `NotFound`, `Unauthorized`, `Forbidden`, `Validation`, `Conflict`, `Internal`.
- Each variant maps to a stable JSON error body and HTTP status code.
- Unexpected errors are logged and returned as `Internal` without leaking internals.

## Background Tasks

- Email notification task runs on an interval and sends queued emails.
- File cleanup task removes orphan file records and storage objects.
- Tasks are spawned as Tokio tasks and share the application state.

## Plugin Loading

- On startup the server scans `PLUGIN_DIR` for native libraries that export the plugin entry point.
- Each plugin is initialized with a host API for logging, configuration, and event subscription.
- Plugin failures are isolated; a crashing plugin does not terminate the server.

## Startup Sequence

1. Load configuration.
2. Initialize tracing.
3. Connect to PostgreSQL and run pending migrations.
4. Load plugins.
5. Build the Axum router and WebSocket manager.
6. Bind to the configured address.
7. Spawn background tasks.

## Shutdown

- SIGTERM triggers a graceful shutdown.
- Open HTTP requests are allowed to complete within a timeout.
- WebSocket connections are closed with a `server_restart` event.
- Plugins receive a shutdown hook before the process exits.
