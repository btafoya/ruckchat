-- Reverse the baseline schema migration.
DROP TABLE IF EXISTS organization_settings;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS message_files;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS reactions;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS dm_members;
DROP TABLE IF EXISTS direct_message_conversations;
DROP TABLE IF EXISTS channel_memberships;
DROP TABLE IF EXISTS channels;
DROP TABLE IF EXISTS organization_memberships;
DROP TABLE IF EXISTS organizations;
DROP TABLE IF EXISTS users;
