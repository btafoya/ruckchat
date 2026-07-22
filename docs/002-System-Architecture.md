
# System Architecture

## Repository

server/
desktop/
mobile/
sdk/
plugins/
shared/
deployment/
docs/

## Technology

Server:
- Rust
- Axum
- Tokio
- SQLx

Desktop:
- Tauri
- React
- TypeScript

Mobile:
- Flutter

Database:
- PostgreSQL

Realtime:
- WebSockets

Storage:
- Local/S3

Search:
- PostgreSQL FTS + pg_trgm

## Principles

- Domain driven design
- Service layer
- Repository pattern
- Event bus
- OpenAPI first
- Thin clients
