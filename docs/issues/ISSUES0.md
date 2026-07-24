# ISSUES0 — @Mentions Support

## Source

> This is meant to be a slack replacement — some features such as mention @ are missing — open

## Research Summary

### Current state

- The composer already has a basic `@` mention trigger (`desktop/src/components/Composer.tsx:92-97`).
- The autocomplete dropdown only shows raw `user_id` UUIDs, filtered by substring match (`Composer.tsx:111-116`).
- Selecting an item inserts `@<user_id>` into the message text (`Composer.tsx:102-109`).
- `MessageItem.tsx` renders message `content` as plain text; `@mentions` are not highlighted or linked.
- The backend `Message` schema (`server/openapi.yaml:3063-3077`) stores only `content`; there is no structured mention metadata or notification-specific field.
- Notifications are driven generically by direct messages and mentions (`desktop/src/hooks/useNotifications.ts`, `desktop/src/components/Settings.tsx:56-58`), but no logic parses `@mentions` to trigger a notification event.

### Gaps

1. **User discovery** — the composer needs a search that resolves by `display_name` and email, not just `user_id`.
2. **Mention rendering** — sent messages should render `@display_name` as a styled, clickable token instead of raw UUID text.
3. **Backend mention extraction** — the server should parse `@<identifier>` on send and emit mention events (push/WebSocket) to targeted users.
4. **Notification routing** — a mention should notify the target user even when they are not actively viewing the conversation.
5. **Thread/DM parity** — mentions should work in channel messages, thread replies, and DM conversations consistently.

### Affected files

- `desktop/src/components/Composer.tsx` — mention autocomplete data source and insertion.
- `desktop/src/components/MessageItem.tsx` — render mention tokens.
- `desktop/src/api/users.ts` (new or existing) — user search endpoint consumer.
- `server/src/services/message.rs` — parse and record mentions.
- `server/src/repositories/notification.rs` (if exists) or WebSocket event bus — emit mention notifications.
- `server/openapi.yaml` — add mention-related schema if needed.

## Open Questions

1. **What should the mention identifier be?**
   - `@display_name` (Slack-style, human-readable).
   - `@user_id` (current implementation, stable but ugly).
   - `@email` prefix or custom username handle.

2. **Where should mention notification state live?**
   - Parse mentions synchronously on message send and push to WebSocket only.
   - Add a `mention` event type to the real-time event bus and a backend queue/persistence layer.

3. **Should mentions be stored as structured metadata or parsed from plain text each time?**
   - Store an array of `mentioned_user_ids` on the message row.
   - Keep plain text only and parse at read/render time.

## Decisions

- Mention identifier: `@display_name` (human-readable, Slack-style).
- Mention persistence: store `mentioned_user_ids` on the message row and emit WebSocket/push events from the backend.
