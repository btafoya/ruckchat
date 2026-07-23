//! Persistent source-to-target ID mapping store backed by SQLite.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension};

use crate::error::Result;

/// SQLite-backed mapping store.
pub struct MappingStore {
    conn: Connection,
}

impl MappingStore {
    /// Opens or creates the mapping store at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let mut store = Self { conn };
        store.ensure_schema()?;
        Ok(store)
    }

    /// Returns an in-memory mapping store for testing.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self { conn };
        store.ensure_schema()?;
        Ok(store)
    }

    fn ensure_schema(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS user_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                email TEXT,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS room_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                rocket_type TEXT,
                ruckchat_type TEXT,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS message_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS file_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                storage_path TEXT,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS emoji_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                shortcode TEXT,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS role_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                name TEXT,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS team_map (
                rocket_id TEXT PRIMARY KEY,
                ruckchat_id TEXT NOT NULL,
                action TEXT
            );
            CREATE TABLE IF NOT EXISTS checkpoints (
                stage TEXT PRIMARY KEY,
                last_id TEXT,
                completed_at TEXT
            );",
        )?;
        Ok(())
    }

    /// Records or updates a user mapping.
    pub fn put_user(
        &self,
        rocket_id: &str,
        ruckchat_id: &str,
        email: Option<&str>,
        action: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO user_map (rocket_id, ruckchat_id, email, action)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 email = excluded.email,
                 action = excluded.action",
            [rocket_id, ruckchat_id, email.unwrap_or(""), action],
        )?;
        Ok(())
    }

    /// Returns the mapped RuckChat user identifier, if any.
    pub fn get_user(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM user_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records or updates a room mapping.
    pub fn put_room(
        &self,
        rocket_id: &str,
        ruckchat_id: &str,
        rocket_type: &str,
        ruckchat_type: &str,
        action: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO room_map (rocket_id, ruckchat_id, rocket_type, ruckchat_type, action)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 rocket_type = excluded.rocket_type,
                 ruckchat_type = excluded.ruckchat_type,
                 action = excluded.action",
            [rocket_id, ruckchat_id, rocket_type, ruckchat_type, action],
        )?;
        Ok(())
    }

    /// Returns the mapped RuckChat room identifier, if any.
    pub fn get_room(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM room_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Returns the type originally recorded for a room mapping.
    pub fn get_room_type(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT rocket_type FROM room_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records or updates a message mapping.
    pub fn put_message(&self, rocket_id: &str, ruckchat_id: &str, action: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO message_map (rocket_id, ruckchat_id, action)
             VALUES (?1, ?2, ?3)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 action = excluded.action",
            [rocket_id, ruckchat_id, action],
        )?;
        Ok(())
    }

    /// Returns the mapped RuckChat message identifier, if any.
    pub fn get_message(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM message_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records or updates a file mapping.
    pub fn put_file(
        &self,
        rocket_id: &str,
        ruckchat_id: &str,
        storage_path: Option<&str>,
        action: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO file_map (rocket_id, ruckchat_id, storage_path, action)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 storage_path = excluded.storage_path,
                 action = excluded.action",
            [rocket_id, ruckchat_id, storage_path.unwrap_or(""), action],
        )?;
        Ok(())
    }

    /// Returns the mapped RuckChat file identifier, if any.
    pub fn get_file(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM file_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records or updates an emoji mapping.
    pub fn put_emoji(
        &self,
        rocket_id: &str,
        ruckchat_id: &str,
        shortcode: &str,
        action: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO emoji_map (rocket_id, ruckchat_id, shortcode, action)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 shortcode = excluded.shortcode,
                 action = excluded.action",
            [rocket_id, ruckchat_id, shortcode, action],
        )?;
        Ok(())
    }

    /// Returns the mapped RuckChat emoji identifier, if any.
    pub fn get_emoji(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM emoji_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records or updates a role mapping.
    pub fn put_role(
        &self,
        rocket_id: &str,
        ruckchat_id: &str,
        name: &str,
        action: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO role_map (rocket_id, ruckchat_id, name, action)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 name = excluded.name,
                 action = excluded.action",
            [rocket_id, ruckchat_id, name, action],
        )?;
        Ok(())
    }

    /// Returns the mapped RuckChat role identifier, if any.
    pub fn get_role(&self, rocket_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM role_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records a completed checkpoint for a stage.
    pub fn put_checkpoint(&self, stage: &str, last_id: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO checkpoints (stage, last_id, completed_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT (stage) DO UPDATE SET
                 last_id = excluded.last_id,
                 completed_at = excluded.completed_at",
            [stage, last_id.unwrap_or("")],
        )?;
        Ok(())
    }

    /// Returns the last checkpoint for a stage, if any.
    pub fn get_checkpoint(&self, stage: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT last_id FROM checkpoints WHERE stage = ?1",
                [stage],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Returns the mapped RuckChat team identifier, if any.
    pub fn get_team(&self, rocket_id: String) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT ruckchat_id FROM team_map WHERE rocket_id = ?1",
                [rocket_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// Records or updates a team mapping.
    pub fn put_team(&self, rocket_id: String, ruckchat_id: String, action: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO team_map (rocket_id, ruckchat_id, action)
             VALUES (?1, ?2, ?3)
             ON CONFLICT (rocket_id) DO UPDATE SET
                 ruckchat_id = excluded.ruckchat_id,
                 action = excluded.action",
            [rocket_id, ruckchat_id, action.to_string()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_round_trip() {
        let store = MappingStore::open_in_memory().unwrap();
        store
            .put_user("u1", "ru1", Some("a@b.com"), "create")
            .unwrap();
        assert_eq!(store.get_user("u1").unwrap(), Some("ru1".into()));
        store
            .put_user("u1", "ru2", Some("a@b.com"), "update")
            .unwrap();
        assert_eq!(store.get_user("u1").unwrap(), Some("ru2".into()));
    }

    #[test]
    fn checkpoint_round_trip() {
        let store = MappingStore::open_in_memory().unwrap();
        store.put_checkpoint("users", Some("last")).unwrap();
        assert_eq!(store.get_checkpoint("users").unwrap(), Some("last".into()));
    }
}
