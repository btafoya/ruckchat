-- Add the unique index on sessions.token_hash that login/logout rely on.
CREATE UNIQUE INDEX IF NOT EXISTS sessions_token_hash_key ON sessions (token_hash);
