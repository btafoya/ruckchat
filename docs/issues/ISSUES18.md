# ISSUES18 — Add Delete Option to Messages

## Source

> Add delete option to message — open

## Research Summary

### Current state

- The backend supports soft-deleting messages via `DELETE /api/v1/messages/{message_id}` (`server/src/handlers/message.rs:76-87`).
- `MessageItem.tsx` currently shows an "Edit" button for the author's own messages, plus quick reactions and reply link, but no "Delete" action (`desktop/src/components/MessageItem.tsx:192-200`).
- The domain model marks a message deleted by setting `deleted_at` and clearing content (`crates/ruckchat-domain/src/message.rs:145-151`).
- `useMessages.ts` exposes `removeMessage(messageId)` to drop a message from local state, but there is no public `deleteMessage` method or API consumer for deletion (`desktop/src/hooks/useMessages.ts:388-390`).
- `MessageContent.tsx` renders `[deleted]` for deleted messages (`desktop/src/components/MessageItem.tsx:129-133`).

### Gaps

1. **No frontend API method** — `desktop/src/api/messages.ts` needs a `deleteMessage(token, messageId)` wrapper.
2. **No hook action** — `useMessages` needs an async `deleteMessage(messageId)` that calls the API and updates local state.
3. **No UI affordance** — `MessageItem` needs a "Delete" button (author-only or admin-only) with a confirmation step.
4. **No real-time deletion broadcast** — other clients should receive a WebSocket event so the deleted message renders as `[deleted]` immediately.
5. **No permission model clarity** — can only authors delete? Can organization managers/server admins delete others' messages?

### Affected files

- `desktop/src/api/messages.ts` — add `delete` method.
- `desktop/src/hooks/useMessages.ts` — add `deleteMessage` action and wire to WebSocket event if one exists.
- `desktop/src/components/MessageItem.tsx` — add delete button and confirmation.
- `server/src/handlers/message.rs` — already has the endpoint; verify OpenAPI docs.
- `server/src/services/message.rs` — verify soft-delete behavior and event emission.
- `server/src/services/events.rs` and `server/src/websocket/bus.rs` — add `message.deleted` event if missing.

## Open Questions

1. **Who can delete?**
   - Author only.
   - Author + organization managers + server admins.

2. **Confirmation behavior**
   - Inline confirmation button (click once to confirm).
   - Modal confirmation dialog.

3. **Thread reply deletion**
   - Should deleting a thread reply remove it from the thread pane immediately?
   - Should deleting the parent message also delete or collapse replies?

4. **Undo**
   - No undo (matches Slack).
   - Short undo toast with server reversal.

## Decisions

- Pending. Author-only deletion is the safest default; needs confirmation UX design.

## Status

Open.
