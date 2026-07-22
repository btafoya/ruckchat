# 014 - Deployment

## Deployment Philosophy

RuckChat is designed to be deployed as a single server process with one PostgreSQL database. The goal is to make self-hosting as simple as running a binary and connecting a database.

## Single Binary Deployment

The server is compiled to a single executable:

```bash
ruckchat-server
```

Run with environment variables:

```bash
DATABASE_URL=postgres://ruckchat:secret@localhost/ruckchat
SESSION_SECRET=$(openssl rand -hex 32)
RUCKCHAT_PORT=3000
./ruckchat-server
```

## Docker Deployment

An official Docker image is provided:

```bash
docker run -e DATABASE_URL=... -e SESSION_SECRET=... -p 3000:3000 ruckchat/server:latest
```

A `docker-compose.yml` example is included in the repository for local and small production deployments.

## Reverse Proxy

Caddy is recommended because it handles HTTPS automatically:

```
ruckchat.example.com {
    reverse_proxy localhost:3000
}
```

Nginx and Traefik are also supported, but Caddy is the documented default.

## Required Infrastructure

- One server capable of running the Rust binary.
- One PostgreSQL 15+ database.
- A reverse proxy for TLS termination.
- (Optional) An SMTP relay for email notifications.
- (Optional) An S3-compatible object store for file storage.

## File Storage

Default storage is the local filesystem in the `FILE_STORAGE_PATH` directory. For production, an S3-compatible store is recommended for durability and backup.

## Database Setup

1. Create the database and user.
2. Set `DATABASE_URL`.
3. Run the server; migrations apply automatically.

For manual migrations:

```bash
cargo sqlx migrate run
```

## Environment Checklist

Before running in production, verify:

- [ ] `SESSION_SECRET` is a strong, unique secret.
- [ ] `DATABASE_URL` points to a persistent PostgreSQL instance.
- [ ] HTTPS is enabled via reverse proxy.
- [ ] `SMTP_*` variables are set if email notifications are required.
- [ ] File storage backend is configured and accessible.
- [ ] Backups are scheduled for the database and file storage.

## Scaling

v1 scales vertically. Horizontal scaling is not supported because WebSocket state and rate limiters are in-memory. If a deployment outgrows a single server, the architecture must be revisited in a later release.

## Updates

- New server versions apply migrations on startup.
- Updates are documented in release notes with any required manual steps.
- A maintenance mode flag can be enabled to drain connections before restart.

## Desktop and Mobile Distribution

- Desktop clients are released through GitHub releases as platform installers.
- Mobile clients are released through the Google Play Store and Apple App Store in post-MVP.
