# ruckchat-migrate

Idempotent, versioned export/import of RuckChat's core domain data as a
single JSON `MigrationData` snapshot. Used by `ruckchat-server`'s CLI
(`migrate export`/`migrate import`) and admin import endpoint, and by the
standalone `rocketchat2ruckchat` migration tool for writing directly to a
target RuckChat PostgreSQL database.
