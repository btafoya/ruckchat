-- Add the unique index on users.email that the baseline schema declares.
-- This migration is idempotent for databases created from earlier iterations.
CREATE UNIQUE INDEX IF NOT EXISTS users_email_key ON users (email);
