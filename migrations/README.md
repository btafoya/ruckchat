# ruckchat-migrations

SQLx migrations for the RuckChat PostgreSQL schema.

## Running migrations

From the workspace root:

```bash
# Apply migrations using sqlx-cli (install with `cargo install sqlx-cli`)
sqlx migrate run --source migrations/migrations --database-url "$DATABASE_URL"
```

From Rust code:

```rust
let pool = sqlx::PgPool::connect(&database_url).await?;
ruckchat_migrations::migrator().run(&pool).await?;
```

## Running integration tests

Integration tests create an isolated PostgreSQL database per test run. Set the
admin connection URL if your database is not on the default Docker port:

```bash
RUCKCHAT_TEST_ADMIN_DATABASE_URL=postgres://user:pass@localhost:5432/postgres \
  cargo test -p ruckchat-migrations
```
