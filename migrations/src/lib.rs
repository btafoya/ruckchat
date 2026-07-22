//! SQLx migrations for RuckChat.
//!
//! The canonical migration files live in `migrations/` adjacent to this crate.
//! Use [`sqlx::migrate!`] from application code to apply them at runtime.

/// Returns a SQLx migrator pointing at the embedded `migrations/` directory.
#[must_use]
pub fn migrator() -> sqlx::migrate::Migrator {
    sqlx::migrate!()
}
