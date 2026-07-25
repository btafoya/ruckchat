# ADR-016: Cursor-Based Message Pagination and Per-Message Unread State

## Status

Accepted — implementing.

## Context

The message list (channel/DM history and thread replies) had no real
scroll model: the server returned messages newest-first via
`LIMIT/OFFSET`, the client rendered that array as-is (newest message at the
*top* of the screen), and "load more" was a manual button that appended
progressively older pages to the *end* of the array. There was no
auto-follow of new messages, no scroll-up infinite pagination, and thread
replies were not paginated at the database level at all —
`MessageService::get_thread_replies` fetched up to 1000 messages via
`list_by_conversation` and filtered by `parent_id` in application code.

A brainstorming session (see `claudedocs/workflow_message-list-reload.md`)
established the target behavior: ascending display order; auto-scroll at
the bottom with a live "new messages" indicator when scrolled up; automatic
scroll-up pagination; no eviction of already-loaded messages; the same
treatment for thread replies; a deep-link "jump to message" for
search/mention navigation that must paginate in both directions from an
arbitrary anchor; and, added after initial planning, a per-message unread
indicator that clears individually as each message is marked read.

Offset-based (`LIMIT/OFFSET`) pagination is well known to skip or duplicate
rows when rows are inserted between page fetches — a real, common case here
since scroll-up pagination now happens while messages are actively arriving
in busy channels. `MessageId::new()` is `Uuid::new_v4()`
(`crates/ruckchat-id/src/lib.rs`) — a random, non-time-sortable id — so a
correct cursor cannot use `id` alone; it must be tied to `created_at`, which
itself is not unique enough alone (concurrent posts, bulk-imported history
can share a timestamp).

## Decision

- **Cursor key is `(created_at, id)`**, compared as a Postgres row value:
  `(created_at, id) < ($cursor_created_at, $cursor_id)` for older,
  `>` for newer. This is gapless and duplicate-free regardless of how many
  messages share a `created_at`, and requires no new column or migration —
  the existing `idx_messages_conversation_id_created_at (conversation_id,
  created_at DESC)` index already supports the range scan; `id` is filtered
  within the (typically tiny) set of same-timestamp rows.
- **Repository layer replaces `list_by_conversation(limit, offset)`** with
  two keyset methods, always returning ascending order:
  - `list_before(conversation_id, before: Option<MessageCursor>, limit)` —
    the `limit` messages immediately older than `before`, or the newest
    `limit` messages if `before` is `None`.
  - `list_after(conversation_id, after: MessageCursor, limit)` — the
    `limit` messages immediately newer than `after`.
  - `MessageCursor { created_at, id }` is a new small struct in
    `crates/ruckchat-domain/src/repositories.rs`, not a domain entity.
  - Thread replies get their own parallel `list_replies_before` /
    `list_replies_after`, scoped by `parent_id` (already indexed via
    `idx_messages_parent_id`), replacing the fetch-1000-and-filter hack
    outright — not kept alongside it.
- **"Around" (deep-link jump) is composed at the service layer**, not a
  third repository method: `MessageService` resolves the anchor via
  `by_id`, then calls `list_before` (exclusive, older half) and
  `list_after` (exclusive, newer half) around it. This reuses the two
  primitives instead of adding a third query shape.
- **`has_more_older`/`has_more_newer` use the existing "returned length ==
  requested limit" heuristic** (the same convention `useMessages.ts`
  already uses for `hasMore` today) rather than an extra `COUNT` query.
  For the initial/newest load (`before: None`), `has_more_newer` is always
  `false` — that call is by definition already at the live tail.
- **HTTP layer takes message ids, not raw cursors.** `GET
  /channels/{id}/messages`, `GET /direct_messages/{id}/messages`, and the
  thread-replies endpoint accept `before_id` / `after_id` / `around_id`
  (message ids) plus `limit`, replacing `Pagination { limit, offset }` on
  these three routes only. The server resolves an id to its
  `(created_at, id)` cursor via one `by_id` lookup; the client never
  constructs or reasons about a cursor tuple directly.
- **`search.rs` and the MCP `get_messages`/`search_messages` tools are
  unaffected** — they keep `Pagination { limit, offset }` as-is. This
  redesign is scoped to interactive channel/DM/thread browsing only.
- **Per-message `is_unread` is a response-only field, never on the shared
  domain `Message`.** `Message` is also the payload for WebSocket broadcasts
  (`message.created`/`message.updated`, sent verbatim to every recipient in
  a conversation) and MCP tool results — baking a caller-relative read flag
  into that struct would leak one recipient's read state to everyone else
  receiving the same broadcast, or misrepresent it for all but one reader.
  Instead, the three history/replies handlers wrap each message in a new
  `MessageWithReadState { #[serde(flatten)] message: Message, is_unread:
  bool }` response DTO, computing `is_unread` via a new thin
  `ReadStateService::unread_ids` passthrough to the existing
  `MessageReadRepository::unread_message_ids` (already used internally by
  `search.rs`'s `is:unread` operator — no new repository method needed).
  Per the existing ADR-015 rule, a caller's own authored messages are never
  unread; that exclusion is applied at the handler call site
  (`m.author_id != caller_id`), mirroring exactly how `search.rs` already
  does it, rather than baking the exclusion into the shared repository
  method.
- **Client keeps a local `Set<messageId>` of unread ids** per open
  conversation, seeded from each page's `is_unread` fields, cleared
  optimistically the instant a message is marked read (the existing
  scroll-into-view read-state batching) and also cleared on incoming
  `read_state.updated` events (cross-tab/cross-device case, already
  broadcast to all of the same user's sessions per ADR-015).
  `MessageItem.tsx` renders a dot/marker for ids in that set, never for the
  caller's own messages.
- **Client tracks an explicit live-tail vs. anchored-history mode.**
  Live-tail (`hasMoreNewer === false`, the normal "just opened the channel"
  state) accepts WebSocket `message.created` events via the existing
  `appendMessage`. Anchored-history mode (after a `jumpToMessage` that
  hasn't yet paginated forward to the real tail) must **not** splice
  WS-delivered messages into a gap between the loaded window's newest
  message and the true tail.
- **No eviction of loaded messages.** Once a page is fetched it stays in
  memory/DOM for the session, matching how Slack/Discord behave in
  practice. Explicitly out of scope; revisit only if it becomes a measured
  problem.
- **Unread-marking on scroll-into-view fires immediately**, no dwell delay
  — a fast scroll-through marks passed-over messages read, matching
  Slack/Discord and avoiding a per-visible-message timer.
- **"New messages" indicator is a pill with a live unread count** (e.g.
  "↓ 3 new messages"), not a generic label or icon-only button.

## Consequences

### Positive
- Scroll-up pagination is correct under concurrent inserts, which offset
  pagination was not — this was a real, not hypothetical, gap given
  auto-loading now happens continuously while messages arrive.
- Thread replies get a real paginated query instead of a 1000-row fetch
  filtered in memory, fixing both a correctness ceiling (a thread with more
  than 1000 total conversation messages could already silently truncate
  replies) and a performance concern.
- `is_unread` reuses an existing, already-tested repository method with no
  new schema or query — the addition is response-shaping and call-site
  filtering only.

### Negative
- The main history endpoints are strictly more complex (three optional id
  params instead of `limit`/`offset`), and the client now has two loading
  modes (live-tail vs. anchored-history) to reason about correctly — this
  is the plan's highest-risk correctness area and needs explicit test
  coverage for the "don't splice WS messages into a gap" rule.
- `is_unread` costs one extra batched repository call per history/replies
  fetch (same query shape `search.rs` already pays for its `is:unread`
  operator).

## Implementation

- `crates/ruckchat-domain/src/repositories.rs` — `MessageCursor`;
  `MessageRepository::list_before`/`list_after`/`list_replies_before`/
  `list_replies_after` replacing `list_by_conversation`.
- `server/src/repositories/message.rs` — SQLx implementations, reusing the
  existing `load_attachments` batch-join for every new query path.
- `server/src/testing.rs` — `MockMessageRepository` in-memory
  implementations.
- `server/src/services/dto.rs` — `MessagePage { messages, has_more_older,
  has_more_newer }`; `MessagePageQuery { before_id, after_id, around_id,
  limit }`.
- `server/src/services/message.rs` — `get_history`/`get_thread_replies`
  rewritten on the cursor methods; anchor composition for `around_id`.
- `server/src/services/read_state.rs` — `ReadStateService::unread_ids`.
- `server/src/handlers/dto.rs` — `MessageWithReadState`,
  `MessagePageResponse`.
- `server/src/handlers/message.rs`/`channel.rs`/`direct_message.rs` —
  updated query extraction and response composition.
- `server/openapi.yaml` — updated paths/schemas for the three affected
  endpoints.
- `desktop/src/api/schema.ts` — regenerated.
- `desktop/src/hooks/useMessages.ts` — `loadOlder`/`loadNewer`/
  `jumpToMessage`, live-tail/anchored-history gating, local unread-id set.
- `desktop/src/components/MessagePane.tsx`/`ThreadPane.tsx` — scroll
  anchoring, visibility tracking, "↓ N new messages" pill, mark-read
  batching.
- `desktop/src/components/MessageItem.tsx` — per-message unread
  dot/marker.
- `server/tests/message.rs` (or a new dedicated test file) — cursor
  pagination correctness, concurrent-insert behavior, `around_id`
  bidirectional fetch, `is_unread` end-to-end.

## Related

- `docs/ADR-006-WebSocket-Real-Time-Events.md`
- `docs/ADR-015-Search-And-Read-State.md` (read-state model and the
  own-messages-never-unread rule this reuses)
- `claudedocs/workflow_message-list-reload.md`
