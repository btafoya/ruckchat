# 016 - Operations

## Configuration

The server reads one YAML configuration file at startup.

- Default locations:
  - Linux: `/etc/ruckchat/ruckchat.yaml`
  - macOS: `/Library/Application Support/RuckChat/ruckchat.yaml`
  - Windows: `%ProgramData%\RuckChat\ruckchat.yaml`
- Override with `--config <path>` for development or non-standard installs.
- Generate a template with `ruckchat-server --init-config [path]`.

The file is read once at startup and is not reloaded automatically. Edit the file and restart the service to apply changes. Validate a file before restarting by loading it from a temporary path:

```bash
ruckchat-server --config /tmp/ruckchat-staging.yaml
```

## Logging

- The server uses `tracing` for structured logging.
- Default log level is `info`, driven by `log_level` in `ruckchat.yaml`.
- Request logs include method, path, status, duration, and authenticated user ID.
- Sensitive fields are redacted from logs.

## Health Checks

- `GET /health` returns `200 OK` when the server is running.
- `GET /health/ready` returns `200 OK` when the database connection pool is healthy and migrations are current.
- Reverse proxies and orchestrators should use `/health/ready` for readiness probes.
- The Docker image includes a `HEALTHCHECK` that probes `GET /` on port `3000`.

## Server CLI

The server binary provides a small set of operational subcommands:

- `ruckchat-server --init-config [PATH]` — write a default `ruckchat.yaml` and
  exit.
- `ruckchat-server --config PATH migrate export --output PATH` — export a
  domain snapshot.
- `ruckchat-server --config PATH migrate import --input PATH [--dry-run]` —
  import a domain snapshot idempotently.

Run with `--help` to see the full CLI.

## Metrics

- v1 exposes basic Prometheus metrics at `GET /metrics` when `METRICS_ENABLED=true`.
- Metrics include:
  - HTTP request count and duration by route.
  - Active WebSocket connections.
  - Database pool usage.
  - Background task execution counts.
- Detailed business metrics are a post-MVP feature.

## Backups

- Database backups are the operator's responsibility.
- Recommended: nightly `pg_dump` or continuous WAL archiving.
- File storage backups mirror the database backup schedule when using local storage.
- Object storage users should enable versioning and cross-region replication.

## Disaster Recovery

- Recovery time objective (RTO): operator-defined; documented in runbooks.
- Recovery point objective (RPO): same as backup frequency.
- Restore procedure:
  1. Restore the PostgreSQL database from backup or import a domain snapshot.
  2. Restore file storage from backup or object-store versioning.
  3. Restart the server and verify `/health/ready`.

Domain snapshots are a lightweight alternative to full database restores for
migrating or cloning an instance. They carry all domain metadata but not file
payloads, so file storage must still be restored separately.

## Alerting

- Recommended alerts:
  - Server process down.
  - Database connection pool exhausted.
  - Disk usage above 80%.
  - Error rate spike (5xx responses).
- Alerting integrations (PagerDuty, Slack webhooks) are post-MVP.

## Maintenance Mode

- `MAINTENANCE_MODE=true` returns `503 Service Unavailable` for non-health endpoints.
- Health endpoints continue to respond so load balancers can verify the process.
- Use maintenance mode during database migrations that require downtime.

## Log Rotation

- When logging to files, logs are rotated daily and retained for 30 days by default.
- Container deployments should forward logs to the host or a log aggregator.

## Capacity Planning

- Vertical scaling is the primary lever in v1.
- Monitor CPU, memory, and PostgreSQL connection usage as user counts grow.
- Plan to revisit architecture if active WebSocket connections exceed tens of thousands on a single host.
