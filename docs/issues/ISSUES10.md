# ISSUES10 — Submitted Message Is Duplicated

## Source

> WHen I submit a message it is duplicated — open

## Research Summary

### Current state

- `useMessages.sendMessage` adds an optimistic pending message with a `pending-${Date.now()}` ID and then replaces it with the server-returned message on success (`desktop/src/hooks/useMessages.ts:204-241`).
- `appendMessage` deduplicates incoming WebSocket `message.created` events against `message.id` before splicing them into the list (`desktop/src/hooks/useMessages.ts:366-371`).
- The hook is bound to a single `conversationId`; it ignores `message.created` events whose `conversation_id` does not match (`desktop/src/hooks/useMessages.ts:349-358`).
- `hasMoreNewer` gates live splicing: when the user has paginated away from the tail, WebSocket messages are intentionally dropped until `loadNewer()` catches up (`desktop/src/hooks/useMessages.ts:359-365`).

### Gaps

1. **Race between optimistic replacement and WebSocket broadcast** — the server may broadcast the newly created message over the WebSocket before the REST response returns, causing `appendMessage` to insert it before the pending message is replaced. The deduplication check then sees a real ID and a pending ID as distinct, leaving both visible.
2. **Pending ID collision** — `pending-${Date.now()}` can collide if two messages are sent within the same millisecond.
3. **Missing deduplication by content/author/timestamp** — if the real message arrives first and the optimistic message is still in the list, the replacement step may append a second copy.
4. **No reproduction test** — there is no unit or integration test that sends two messages rapidly or asserts no duplicates after a concurrent WebSocket delivery.

### Affected files

- `desktop/src/hooks/useMessages.ts` — optimistic send, `appendMessage`, and deduplication logic.
- `desktop/src/hooks/useWebSocket.ts` — message event ordering and delivery timing.
- `desktop/src/components/Shell.test.tsx` — existing tests that cover message sending.
- `server/src/websocket/bus.rs` — broadcast timing for `message.created`.

## Open Questions

1. **Should the fix be client-side, server-side, or both?**
   - Client-side only: keep a set of recently sent content/timestamps and ignore matching WebSocket messages until the REST response resolves.
   - Server-side only: delay the broadcast by a few milliseconds (unreliable).
   - Both: client deduplication plus a deterministic temporary ID keyed by conversation + content + nonce.

2. **Should the pending message be replaced or removed?**
   - Replace in place (current approach) once the real ID arrives.
   - Remove the pending message immediately on send and rely entirely on the WebSocket event.

3. **Is the duplication reproducible on slow networks, fast double-sends, or both?**
   - Needs a targeted test to confirm the trigger.

## Decisions

- Client-side fix: track recently sent content per pending optimistic message and, when a `message.created` WebSocket event arrives from the current user with matching content, replace the pending item in place rather than appending. The REST-response path also checks whether the real message already exists and drops the pending copy if so. Pending IDs now include a monotonic counter to avoid `Date.now()` collisions.

## Status

Complete.
