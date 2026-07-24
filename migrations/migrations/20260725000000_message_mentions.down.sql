DROP INDEX IF EXISTS idx_messages_mentioned_user_ids;

ALTER TABLE messages
    DROP COLUMN IF EXISTS mentioned_user_ids;
