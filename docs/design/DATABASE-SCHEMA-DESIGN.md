# RuckChat v1 Database Schema Design

## 1. Design Goals

- Single PostgreSQL database for all tenants.
- Strong tenant isolation through foreign keys and application-level authorization.
- Support for messaging, channels, direct messages, files, sessions, search, and settings.
- All tables use UUID primary keys and `timestamptz`.

## 2. Conventions

- Table names: plural, `snake_case`.
- Primary keys: `UUID`, default `gen_random_uuid()`.
- Tenant column: `organization_id` where applicable.
- Soft deletes: `deleted_at` timestamp, not a boolean.
- Timestamps: `created_at`, `updated_at` on mutable entities.
- Foreign keys include explicit `ON DELETE` behavior.
- Indexes are added where query patterns or unique constraints require them.

## 3. Entity Relationship Overview

```
users
  │
  ├───< organizations.owner_id
  │
  ├───< organization_memberships.user_id
  │       │
  │       └───> organizations
  │
  ├───< channels.created_by
  │       │
  │       └───> organizations
  │
  ├───< channel_memberships.user_id
  │       │
  │       └───> channels
  │
  ├───< direct_message_conversations (via dm_members.user_id)
  │       │
  │       └───> organizations
  │
  ├───< messages.author_id
  │       │
  │       ├───> channels (when conversation_type = 'channel')
  │       └───> direct_message_conversations (when conversation_type = 'dm')
  │
  ├───< reactions.user_id
  │       │
  │       └───> messages
  │
  ├───< files.uploaded_by
  │       │
  │       └───> organizations
  │
  └───< sessions.user_id
```

## 4. Schema

### 4.1 users

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.2 organizations

```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.3 organization_memberships

```sql
CREATE TABLE organization_memberships (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, organization_id)
);
```

### 4.4 channels

```sql
CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    topic TEXT,
    purpose TEXT,
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    UNIQUE (organization_id, name)
);
```

### 4.5 channel_memberships

```sql
CREATE TABLE channel_memberships (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, channel_id)
);
```

### 4.6 direct_message_conversations

```sql
CREATE TABLE direct_message_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.7 dm_members

```sql
CREATE TABLE dm_members (
    conversation_id UUID NOT NULL REFERENCES direct_message_conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (conversation_id, user_id)
);
```

### 4.8 messages

```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL,
    conversation_type TEXT NOT NULL CHECK (conversation_type IN ('channel', 'dm')),
    parent_id UUID REFERENCES messages(id),
    author_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    content_tsv TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', content)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

**Indexes:**

```sql
CREATE INDEX idx_messages_conversation_created
    ON messages(conversation_id, conversation_type, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_messages_parent_id
    ON messages(parent_id)
    WHERE parent_id IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX idx_messages_content_tsv
    ON messages USING GIN (content_tsv);
```

### 4.9 reactions

```sql
CREATE TABLE reactions (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (message_id, user_id, emoji)
);
```

### 4.10 files

```sql
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    file_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    thumbnail_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.11 message_attachments

```sql
CREATE TABLE message_attachments (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    PRIMARY KEY (message_id, file_id)
);
```

### 4.12 sessions

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
```

### 4.13 organization_settings

```sql
CREATE TABLE organization_settings (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    max_file_size_bytes BIGINT NOT NULL DEFAULT 26214400,
    total_storage_quota_bytes BIGINT NOT NULL DEFAULT 10737418240,
    allow_member_channel_creation BOOLEAN NOT NULL DEFAULT TRUE,
    default_message_edit_window_minutes INT NOT NULL DEFAULT 1440,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.14 user_preferences

```sql
CREATE TABLE user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    email_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    desktop_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    theme TEXT NOT NULL DEFAULT 'system' CHECK (theme IN ('light', 'dark', 'system')),
    timezone TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.15 email_notification_queue

```sql
CREATE TABLE email_notification_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK (type IN ('mention', 'dm', 'thread_reply')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT
);

CREATE INDEX idx_email_queue_unsent
    ON email_notification_queue(created_at)
    WHERE sent_at IS NULL AND failed_at IS NULL;
```

## 5. Polymorphic Conversation Reference

The `messages` table uses `conversation_id` + `conversation_type` instead of two nullable foreign keys. The application enforces referential integrity based on `conversation_type`:

- `channel`: `conversation_id` references `channels.id`.
- `dm`: `conversation_id` references `direct_message_conversations.id`.

**Rationale:** It avoids nullable foreign-key columns and keeps the message table simple. The tradeoff is no database-level foreign-key enforcement on `conversation_id`; the repository layer validates the target exists before insertion.

## 6. Search Design

- `messages.content_tsv` is a generated `tsvector` using English full-text search.
- Search queries use `@@ to_tsquery('english', :query)` against `content_tsv`.
- Results are filtered by organization and conversation visibility.
- Ranking is added with `ts_rank_cd`.

Example query:

```sql
SELECT m.*, ts_rank_cd(m.content_tsv, query) AS rank
FROM messages m,
     to_tsquery('english', 'deployment & docker') query
WHERE m.content_tsv @@ query
  AND m.conversation_type = 'channel'
  AND m.conversation_id = :channel_id
  AND m.deleted_at IS NULL
ORDER BY rank DESC, m.created_at DESC
LIMIT 20;
```

## 7. Indexes

| Index | Table | Purpose |
|-------|-------|---------|
| Primary key | all | Unique row identity |
| `users_email` | users | Unique email lookup |
| `organizations_slug` | organizations | Unique slug lookup |
| `(organization_id, name)` | channels | Unique channel name per org |
| `idx_messages_conversation_created` | messages | Fetch recent channel/DM messages |
| `idx_messages_parent_id` | messages | Fetch thread replies |
| `idx_messages_content_tsv` | messages | Full-text search |
| `idx_sessions_user_id` | sessions | List/ invalidate user sessions |
| `idx_sessions_token_hash` | sessions | Session lookup by token hash |
| `idx_email_queue_unsent` | email_notification_queue | Poll for pending emails |

## 8. Data Integrity Rules

1. An organization must always have at least one owner. Enforced by preventing the last owner from leaving or being demoted.
2. A private channel requires channel membership for read access.
3. A public channel requires membership for posting.
4. DM membership is immutable in v1.
5. A user can only edit or delete their own messages unless they are admin/owner.
6. Reactions are unique per `(message_id, user_id, emoji)`.
7. File storage quota is checked at upload time in the service layer.

## 9. Migration Order

1. `users`
2. `organizations`
3. `organization_memberships`
4. `sessions`
5. `user_preferences`
6. `organization_settings`
7. `channels`
8. `channel_memberships`
9. `direct_message_conversations`
10. `dm_members`
11. `messages`
12. `reactions`
13. `files`
14. `message_attachments`
15. `email_notification_queue`

## 10. Quotas and Defaults

| Setting | Default | Source |
|---------|---------|--------|
| Max file size | 25 MB | `organization_settings.max_file_size_bytes` |
| Org storage quota | 10 GB | `organization_settings.total_storage_quota_bytes` |
| Message edit window | 24 hours | `organization_settings.default_message_edit_window_minutes` |
| Session lifetime | 30 days | Application configuration |
| Max DM members | 8 | Application configuration |

## 11. Files Produced

- `docs/design/DATABASE-SCHEMA-DESIGN.md` (this file)
- `docs/design/ARCHITECTURE-DESIGN.md`
- `docs/design/OPENAPI-DESIGN.md`
