//! Integration tests that apply the baseline migrations to a PostgreSQL database
//! and verify that all expected tables, columns, and constraints exist.

use sqlx::{Pool, Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};
use uuid::Uuid;

/// Database connection details used for the test admin connection.
struct AdminConnection {
    user: String,
    password: String,
    host: String,
    port: u16,
}

impl AdminConnection {
    /// Reads admin connection details from the environment or uses the Docker
    /// Postgres default (`postgres:postgres@localhost:5445`).
    fn from_env() -> Self {
        let url = std::env::var("RUCKCHAT_TEST_ADMIN_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5445/postgres".into());
        Self::parse(&url)
    }

    /// Parses a PostgreSQL URL into connection details. Only the path is used
    /// to detect the maintenance database; it is ignored when constructing per-test URLs.
    fn parse(url: &str) -> Self {
        let rest = url
            .strip_prefix("postgres://")
            .expect("admin URL must use postgres:// scheme");
        let (credentials, location) = rest.split_once('@').expect("admin URL must contain @");
        let (user, password) = credentials
            .split_once(':')
            .expect("admin URL must contain user:password");
        let (host, _path) = location.split_once('/').unwrap_or((location, "postgres"));
        let (host, port) = host.split_once(':').unwrap_or((host, "5432"));
        Self {
            user: user.into(),
            password: password.into(),
            host: host.into(),
            port: port.parse().expect("port must be numeric"),
        }
    }

    /// Returns a URL connecting to `database` with these credentials.
    fn url(&self, database: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, database
        )
    }
}

/// Creates an isolated test database, runs the migration, yields a pool to it,
/// and drops the database when the returned guard is dropped.
async fn with_test_db<F, Fut>(test: F)
where
    F: FnOnce(Pool<Postgres>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let db_name = format!("ruckchat_test_{}", Uuid::new_v4().simple());
    let admin = AdminConnection::from_env();
    let test_url = admin.url(&db_name);

    Postgres::create_database(&test_url)
        .await
        .expect("create test database");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&test_url)
        .await
        .expect("connect to test database");

    let migrator = ruckchat_migrations::migrator();
    migrator.run(&pool).await.expect("apply migrations");

    test(pool.clone()).await;

    pool.close().await;
    let _ = Postgres::drop_database(&test_url).await;
}

#[tokio::test]
async fn baseline_tables_exist() {
    with_test_db(|pool| async move {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = 'public'
             ORDER BY table_name",
        )
        .fetch_all(&pool)
        .await
        .expect("fetch tables");

        let names: Vec<String> = rows.into_iter().map(|r| r.0).collect();
        for expected in [
            "channels",
            "channel_memberships",
            "direct_message_conversations",
            "dm_members",
            "files",
            "message_files",
            "messages",
            "organization_memberships",
            "organization_settings",
            "organizations",
            "reactions",
            "sessions",
            "users",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "missing table: {expected}"
            );
        }
    })
    .await;
}

#[tokio::test]
async fn messages_table_has_full_text_search() {
    with_test_db(|pool| async move {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'messages' AND column_name = 'content_tsv'
            )",
        )
        .fetch_one(&pool)
        .await
        .expect("query content_tsv column");
        assert!(row.0, "content_tsv column should exist");
    })
    .await;
}

#[tokio::test]
async fn users_email_uniqueness_is_enforced() {
    with_test_db(|pool| async move {
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, display_name, password_hash)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind("alice@example.com")
        .bind("Alice")
        .bind("hash")
        .execute(&pool)
        .await
        .expect("insert first user");

        let result = sqlx::query(
            "INSERT INTO users (id, email, display_name, password_hash)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind("alice@example.com")
        .bind("Alice Two")
        .bind("hash")
        .execute(&pool)
        .await;

        assert!(result.is_err(), "duplicate email should be rejected");
    })
    .await;
}

#[tokio::test]
async fn organization_slug_uniqueness_is_enforced() {
    with_test_db(|pool| async move {
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, display_name, password_hash)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind("orgowner@example.com")
        .bind("Owner")
        .bind("hash")
        .execute(&pool)
        .await
        .expect("insert owner");

        let org_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organizations (id, name, slug, owner_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(org_id)
        .bind("Acme")
        .bind("acme")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("insert organization");

        let result = sqlx::query(
            "INSERT INTO organizations (id, name, slug, owner_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind("Acme Two")
        .bind("acme")
        .bind(user_id)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "duplicate slug should be rejected");
    })
    .await;
}
