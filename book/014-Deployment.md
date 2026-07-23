# 014 - Deployment

## Deployment Philosophy

RuckChat is designed to be deployed as a single server process with one PostgreSQL database. The goal is to make self-hosting as simple as running a binary, placing a single YAML configuration file, and connecting a database.

## Single Binary Deployment

The server is compiled to a single executable:

```bash
ruckchat-server --config /etc/ruckchat/ruckchat.yaml
```

Generate a configuration file on first install:

```bash
ruckchat-server --init-config /etc/ruckchat/ruckchat.yaml
```

Edit the generated file to set the PostgreSQL URL, base URL, and plugin directory, then start the service.

## systemd Deployment

```ini
[Unit]
Description=RuckChat server
After=network.target postgresql.service

[Service]
Type=exec
User=ruckchat
Group=ruckchat
ExecStart=/usr/local/bin/ruckchat-server --config /etc/ruckchat/ruckchat.yaml
Restart=on-failure
ReadWritePaths=/var/lib/ruckchat/plugins

[Install]
WantedBy=multi-user.target
```

Place the configuration file at `/etc/ruckchat/ruckchat.yaml` and ensure the `ruckchat` user can read it.

## Docker Deployment

Mount the configuration file into the container:

```bash
docker run -v /opt/ruckchat/ruckchat.yaml:/etc/ruckchat/ruckchat.yaml:ro \
  -p 3000:3000 ruckchat/server:latest
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
2. Set `database.url` in `/etc/ruckchat/ruckchat.yaml`.
3. Run the server; migrations apply automatically.

For manual migrations:

```bash
cargo sqlx migrate run --source migrations/migrations
```

## Environment Checklist

Before running in production, verify:

- [ ] `/etc/ruckchat/ruckchat.yaml` exists and is readable by the server user.
- [ ] `database.url` points to a persistent PostgreSQL instance.
- [ ] `base_url` matches the public HTTPS address served by the reverse proxy.
- [ ] HTTPS is enabled via reverse proxy.
- [ ] SMTP settings are configured if email notifications are required.
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
