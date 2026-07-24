# ISSUES0 — @Mentions Support

## Source

> This is meant to be a slack replacement — some features such as mention @ are missing — open

## Research Summary

### Current state

- The composer uses Tiptap and stores message content as ProseMirror JSON.
- `@` mention autocomplete resolves by `display_name` and email using the new
  `GET /organizations/{id}/members/search` endpoint.
- Selecting an item inserts a Tiptap `mention` node with `id` (user_id) and
  `label` (display_name) attributes.
- `MessageItem.tsx` renders ProseMirror JSON via `MessageContent.tsx`; mention
  nodes render as styled `@display_name` chips.
- The backend `Message` schema and `messages` table store `mentioned_user_ids`.
- The message service extracts mention IDs from ProseMirror JSON on send/edit,
  validates that they are organization members, stores the resolved set, and
  emits targeted `ServerEvent::Mention` events over the WebSocket event bus.
- Web Push mention targeting now uses `mentioned_user_ids` instead of parsing
  plain-text `@user_id` tokens.

### Gaps resolved

1. ✅ User discovery — organization member search by display name/email.
2. ✅ Mention rendering — styled, first-class mention nodes in the message renderer.
3. ✅ Backend mention extraction — ProseMirror JSON traversal, validation, and storage.
4. ✅ Notification routing — targeted WebSocket mention events and Web Push filtering.
5. ✅ Thread/DM parity — extraction runs for channel messages, thread replies, and DMs.

### Affected files

- `desktop/src/components/Composer.tsx` — Tiptap mention extension and autocomplete.
- `desktop/src/components/MentionList.tsx` — mention suggestion list component.
- `desktop/src/components/MessageContent.tsx` — ProseMirror/mention renderer.
- `desktop/src/components/MessageItem.tsx` — uses `MessageContent` for rendering.
- `desktop/src/hooks/useMessages.ts` — optimistic messages include `mentioned_user_ids`.
- `desktop/src/api/organizations.ts` — member search API consumer.
- `server/src/services/message.rs` — mention extraction, validation, event emission.
- `server/src/services/events.rs` — `ServerEvent::Mention` and `publish_mention`.
- `server/src/websocket/bus.rs`, `server/src/plugins/bus.rs`, `server/src/testing.rs` —
  `publish_mention` implementations.
- `server/src/services/web_push.rs` — mention targeting via `mentioned_user_ids`.
- `server/src/services/organization.rs`, `server/src/handlers/organization.rs` —
  member search.
- `crates/ruckchat-domain/src/message.rs` — `mentioned_user_ids` field.
- `server/src/repositories/message.rs` — read/write `mentioned_user_ids`.
- `server/openapi.yaml` — `mentioned_user_ids` on `Message`; member search endpoint.
- `migrations/migrations/20260725000000_message_mentions.*.sql` — database column/index.

## Decisions

- Mention identifier: `@display_name` rendered to users; canonical storage is the
  user ID inside the Tiptap `mention` node `attrs.id`.
- Mention persistence: store `mentioned_user_ids` on the message row; do not
  re-parse from text on read.
- Mention validation: only allow mentions of users who are members of the
  conversation's organization.
- Real-time delivery: `ServerEvent::Mention { user_id, message }` sent only to the
  mentioned user via `broadcast_to_users`.
- Web Push: filter channel subscriptions by `mentioned_user_ids` and current
  membership, excluding the author.

## Status

✅ Complete as of commit `ba9ca30`.
