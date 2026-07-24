ALTER TABLE messages
    ADD COLUMN mentioned_user_ids UUID[] NOT NULL DEFAULT '{}';

CREATE INDEX idx_messages_mentioned_user_ids
    ON messages USING GIN (mentioned_user_ids);
