DROP INDEX IF EXISTS idx_users_deactivated_at;
ALTER TABLE users DROP COLUMN IF EXISTS deactivated_at;
