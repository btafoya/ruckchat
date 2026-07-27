-- Parent-channel link, used to preserve RocketChat discussion → parent room
-- relationships during migration. Storage only; no UI or authorization logic
-- reads this yet.
ALTER TABLE channels ADD COLUMN parent_channel_id UUID REFERENCES channels(id) ON DELETE SET NULL;
