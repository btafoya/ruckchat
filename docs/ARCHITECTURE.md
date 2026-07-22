# Architecture

Server:
- Rust
- Axum
- Tokio
- SQLx
- PostgreSQL

Clients:
- Tauri + React
- Flutter

Deployment:
- Single executable
- One PostgreSQL database
- Reverse proxy (Caddy recommended)
