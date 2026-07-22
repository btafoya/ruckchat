# ADR 004: Database Migration Tooling

## Status

Accepted

## Context

RuckChat uses PostgreSQL as its sole persistent store. The schema is relational
and evolves with the domain model. We need a migration system that:

- is version controlled alongside the Rust code,
- runs automatically at server startup and in CI,
- provides reversible down migrations,
- supports compile-time verification of SQL in Rust tests.

## Decision

Use SQLx migrations embedded in a `migrations` crate.

- Migration files live in `migrations/migrations/` and follow SQLx naming:
  `YYYYMMDD_HHMMSS_description.sql` with matching `.down.sql` files.
- The `migrations` crate exposes a `migrator()` function that returns a
  `sqlx::migrate::Migrator`, allowing the server to apply migrations at runtime.
- Integration tests create an isolated PostgreSQL database per test, run the
  migrations, and verify tables, columns, and constraints.

## Consequences

- The server depends only on the migrator crate, not on raw SQL files.
- CI must provide a PostgreSQL service for integration tests.
- Developers need a local PostgreSQL instance (or Docker container) on port
  5445 by default, configurable via `RUCKCHAT_TEST_ADMIN_DATABASE_URL`.
