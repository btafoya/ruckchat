# 015 - Migration

## Migration Tooling

Database migrations are authored in plain SQL and applied with SQLx. The `migrations/` directory is a dedicated crate in the workspace that contains migration files and the migration runner.

## Migration File Format

Each migration consists of two files:

```
migrations/
├── YYYYMMDD_HHMMSS_description.up.sql
└── YYYYMMDD_HHMMSS_description.down.sql
```

- Timestamps are UTC and ordered lexicographically.
- Up migrations apply schema changes.
- Down migrations revert schema changes.
- Every up migration must have a matching down migration.

## Applying Migrations

Migrations apply automatically when the server starts unless `MIGRATIONS_AUTO_RUN` is set to `false`.

Manual application:

```bash
cargo sqlx migrate run
```

Revert the most recent migration:

```bash
cargo sqlx migrate revert
```

## Migration Rules

- Migrations must be idempotent when possible.
- Destructive changes (column drops, table renames) require careful review and a data-backup step.
- New tables and columns include sensible defaults or are nullable to avoid locking large tables during deployment.
- Indexes are created concurrently when adding them to existing large tables:
  ```sql
  CREATE INDEX CONCURRENTLY idx_name ON table(column);
  ```
- Foreign keys include `ON DELETE` behavior explicitly.

## Migration Checklist

Before committing a migration:

- [ ] Up and down files are present.
- [ ] Migration has been run against a fresh database.
- [ ] Migration has been reverted and re-applied successfully.
- [ ] `cargo sqlx prepare` has been run if query metadata changed.
- [ ] Corresponding application code compiles and tests pass.

## Migration Version Table

SQLx maintains a `_sqlx_migrations` table in the database to track applied migrations. This table is managed by SQLx and should not be edited manually.

## Cross-Version Compatibility

- The server supports running against the database version it expects.
- Running a newer server binary against an older database is not supported; migrations must be applied first.
- Older server binaries against newer databases may fail; rolling updates should apply migrations before deploying new code.

## Data Migration

- Bulk data migrations are performed with dedicated SQL scripts in `migrations/scripts/`.
- Data migrations are reviewed separately from schema migrations.
- Large data migrations run outside of the automated migration flow to avoid long transactions.
