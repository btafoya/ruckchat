# Workflow: Chat Message List Reload

Source: requirements finalized in this session's `/sc:brainstorm` pass (see chat
history). This document is a **plan only** — no code has been written or
changed as part of producing it.

## Summary of what's being built

Replace today's message-list model — newest-first array, no scroll
management, manual "Load more" button, offset-based `LIMIT/OFFSET` pagination
— with:

1. Ascending (oldest→newest) display order, main list and thread pane alike.
2. Auto-follow at the bottom; a live-updating "↓ N new messages" pill when
   scrolled up.
3. Automatic scroll-up pagination (no button), scroll-anchored so prepending
   older messages doesn't move the viewport.
4. Cursor-based pagination keyed on `(created_at, id)` — required because
   `MessageId::new()` is `Uuid::new_v4()` (confirmed in
   `crates/ruckchat-id/src/lib.rs:34`), i.e. **not** time-sortable, so `id`
   alone can never be a valid cursor and `created_at` alone can collide.
5. Deep-link "jump to message" (search/mentions) that loads a window
   anchored on an arbitrary message and paginates in both directions from
   there.
6. Read-state marked immediately on scroll-into-view (no dwell delay),
   reusing the existing read-state endpoints.
7. No eviction of already-loaded messages (explicitly out of scope per the
   brainstorm — revisit only if it becomes a measured problem).

This is an **architecture-affecting change** (new pagination model replaces
`Pagination { limit, offset }` for message history) — per `CLAUDE.md`, it
needs a new ADR before implementation, not just code changes.

## Current state (verified in code, not assumed)

- `server/src/repositories/message.rs`: `list_by_conversation` and `search`
  use `ORDER BY created_at DESC LIMIT $2 OFFSET $3`.
- `server/src/services/message.rs::get_thread_replies`: fetches up to 1000
  messages via `list_by_conversation` and filters by `parent_id` **in
  application code** — not a real paginated query.
- `desktop/src/hooks/useMessages.ts`: `refresh()` loads page 0 as-is (DESC,
  newest first); `loadMore()` fetches the next offset page and **appends** it
  to the end of the array — i.e. today the newest message renders at the
  top and "load more" makes progressively older messages appear at the
  bottom. No scroll listeners, no auto-scroll, no visibility tracking exist
  anywhere in `MessagePane.tsx` today.
- Existing index `idx_messages_conversation_id_created_at ON messages
  (conversation_id, created_at DESC)` (baseline migration) already supports
  a `(conversation_id, created_at)` range scan; it does not include `id`, so
  Phase 1 must evaluate whether a covering `(conversation_id, created_at,
  id)` index is worth adding (likely yes, cheap, avoids a tie-break sort
  step) — a call for the implementer, not a hard requirement.
- `idx_messages_parent_id` already exists, so a real thread-replies query
  (`WHERE parent_id = $1 AND (created_at, id) > cursor ...`) will be
  efficient once written.
- Read-state: `POST /channels/{id}/read` / `POST /direct_messages/{id}/read`
  already exist (`ReadStateService`) and accept a list of message ids —
  reusable as-is for the new "mark read on visibility" trigger, just called
  more granularly/batched from the client.

## Proposed API contract (to be finalized in Phase 0's ADR, not frozen here)

Replace the `limit`/`offset` query params on `GET /channels/{id}/messages`
and `GET /direct_messages/{id}/messages` with:

- `before_id` (optional, message id) — return the `limit` messages
  immediately older than this message, ascending order.
- `after_id` (optional, message id) — return the `limit` messages
  immediately newer than this message, ascending order.
- `around_id` (optional, message id) — return roughly `limit/2` older,
  the message itself, and `limit/2` newer, ascending order.
- none of the three — return the newest `limit` messages, ascending order
  (today's "open a channel" case).
- `limit` — unchanged default 50.

Server resolves `before_id`/`after_id`/`around_id` to their `(created_at,
id)` cursor internally via one extra lookup; the client only ever deals in
message ids, never in raw cursor tuples. Response also includes
`has_more_older` / `has_more_newer` booleans so the client knows which
direction(s) still have pages without a length-vs-limit heuristic.

Same shape applies to a new/rewritten thread-replies endpoint
(`GET /messages/{parent_id}/replies` or wherever it currently lives).

`search.rs` and the MCP `get_messages` tool are **out of scope** — they keep
`Pagination { limit, offset }` as-is; only channel/DM history and thread
replies move to cursor pagination.

## Phases

### Phase 0 — ADR and contract sign-off
- **Task:** Write `docs/ADR-016-Cursor-Based-Message-Pagination.md` (next
  available ADR number — confirm against current `docs/ADR-*.md` listing)
  covering: why offset pagination is being replaced, the cursor key
  choice and why (`Uuid::new_v4()` non-orderability), the `before/after/around`
  query contract, the live-tail-vs-anchored-history client state model
  (see Phase 5), and explicitly noting search/MCP are unaffected.
- **Depends on:** nothing.
- **Blocks:** everything else.
- **Checkpoint:** ADR reviewed/accepted before any code changes land.

### Phase 1 — Repository layer
- **Files:** `crates/ruckchat-domain/src/repositories.rs` (trait),
  `server/src/repositories/message.rs` (SQLx impl), `server/src/testing.rs`
  (`MockMessageRepository`).
- **Tasks:**
  - Add cursor-based methods to `MessageRepository`, e.g.
    `list_latest`, `list_before`, `list_after`, `list_around` (exact
    signatures per the Phase 0 ADR).
  - Add a dedicated, properly paginated thread-replies query (replacing the
    fetch-1000-then-filter hack in the service layer) — likely
    `list_replies_before` / `list_replies_after` / `list_replies_latest`
    scoped by `parent_id`.
  - Keep the existing `attachments` batch-join logic (from the prior file-
    attachment fix) working for every new query path — don't regress it.
  - Update `MockMessageRepository` to implement the new trait methods
    in-memory (sorted by `(created_at, id)`) so service-layer unit tests
    keep working without a database.
  - Evaluate/add the `(conversation_id, created_at, id)` covering index
    migration if query plans show it's worth it.
- **Depends on:** Phase 0.
- **Checkpoint:** `cargo test -p ruckchat-domain` and repository-level
  tests pass; a quick `EXPLAIN ANALYZE` check on the new queries against a
  seeded large-ish table confirms index usage.

### Phase 2 — Service layer
- **Files:** `server/src/services/message.rs`.
- **Tasks:**
  - Replace `get_history`'s offset pagination with the new cursor methods,
    preserving existing authorization checks (`require_can_read`).
  - Rewrite `get_thread_replies` to call the new dedicated repository
    query instead of fetching 1000 and filtering in memory.
  - Return `has_more_older`/`has_more_newer` alongside the message list
    (new service-level DTO, or extend the existing return type).
- **Depends on:** Phase 1.
- **Checkpoint:** `server/src/services/message.rs`'s existing unit tests
  updated and passing against `MockMessageRepository`.

### Phase 3 — HTTP handlers & OpenAPI
- **Files:** `server/src/handlers/message.rs` / `channel.rs` /
  `direct_message.rs` (wherever the message-list and thread-replies routes
  live), `server/src/services/dto.rs`, `server/openapi.yaml`.
- **Tasks:**
  - New query-param DTO (`before_id`/`after_id`/`around_id`/`limit`)
    replacing `Pagination` on the affected routes only.
  - Update response schemas to include `has_more_older`/`has_more_newer`.
  - Update `server/openapi.yaml` for every touched endpoint.
- **Depends on:** Phase 2.
- **Checkpoint:** `cargo check`/`clippy` clean; OpenAPI diff reviewed for
  correctness against the Phase 0 ADR contract.

### Phase 4 — Desktop schema regen
- **Files:** `desktop/src/api/schema.ts` (generated).
- **Tasks:** `pnpm dlx openapi-typescript ../server/openapi.yaml -o
  src/api/schema.ts` per the documented regeneration command.
- **Depends on:** Phase 3.
- **Checkpoint:** `pnpm typecheck` in `desktop/` still passes (expected to
  fail until Phase 5's call-site updates land — that's fine, sequencing
  note only).

### Phase 5 — Client data layer
- **Files:** `desktop/src/hooks/useMessages.ts`, thread-replies loading
  logic (currently inline in `useMessages.ts` via `loadThreadReplies` /
  `threadReplies` — consider whether it should become its own hook given
  it now needs its own cursor state).
- **Tasks:**
  - Replace `offset`/`loadMore` state with: `hasMoreOlder`, `hasMoreNewer`,
    `loadOlder()`, `loadNewer()`, `jumpToMessage(messageId)`.
  - Introduce an explicit **"live-tail" vs "anchored-history" mode** flag:
    live-tail (normal open-channel state, `hasMoreNewer === false`) accepts
    WebSocket `message.created` events via the existing `appendMessage`;
    anchored-history mode (after a `jumpToMessage` that isn't yet caught up
    to the real tail) must **not** splice WS-delivered messages into a gap
    — this is the trickiest correctness point in the whole plan and needs
    explicit test coverage.
  - Messages array is always kept in ascending order end-to-end (fetch,
    prepend, append) — no reversal needed in rendering.
  - Track an unread-since-scrolled-up counter for the "↓ N new messages"
    pill.
- **Depends on:** Phase 4.
- **Checkpoint:** `desktop/src/components/*.test.tsx` covering the hook
  (existing `Composer.test.tsx`/`MessagePane.test.tsx` plus new hook-level
  tests) pass.

### Phase 6 — Client UI layer
- **Files:** `desktop/src/components/MessagePane.tsx`,
  `desktop/src/components/ThreadPane.tsx`, a new small pill/indicator
  component, `desktop/src/components/SearchResultsPage.tsx` (or wherever
  mention/search navigation currently routes to a conversation).
- **Tasks:**
  - Top sentinel + `IntersectionObserver` (or scroll-position threshold)
    to trigger `loadOlder()`.
  - Scroll-anchoring on prepend: capture `scrollHeight` before the DOM
    update and restore relative `scrollTop` after, so the viewport doesn't
    jump when older messages are inserted above.
  - Near-bottom detection to decide auto-scroll vs pill-on-new-message.
  - "↓ N new messages" pill component, tap-to-jump-to-bottom.
  - Per-message visibility → immediate read-state call (batched/throttled
    per Phase 7, not one HTTP call per message).
  - Wire search-result/mention navigation to call `jumpToMessage` with the
    target id, and implement the "briefly highlight" visual on arrival.
  - Apply the same sentinel/anchoring/pill treatment to `ThreadPane.tsx`.
- **Depends on:** Phase 5.
- **Checkpoint:** manual verification via the `run` skill or Playwright
  against the local Docker stack: open a channel, scroll up through several
  pages, send a message from a second session and confirm pill behavior,
  click a search result for an old message and confirm anchor+highlight.

### Phase 7 — Read-state batching
- **Files:** `desktop/src/hooks/useReadState.ts` or wherever read-state
  calls originate, `MessagePane.tsx`/`ThreadPane.tsx` visibility wiring
  from Phase 6.
- **Tasks:** debounce/batch visible-message ids over a short window (e.g.
  a trailing-edge debounce of a few hundred ms) before calling the existing
  `POST /channels/{id}/read` / `POST /direct_messages/{id}/read` endpoints,
  so a fast scroll-through doesn't fire one request per message.
- **Depends on:** Phase 6.
- **Checkpoint:** network tab / integration test confirms batching (N
  visible messages → one request, not N requests) during a fast scroll.

### Phase 8 — Tests
- **Backend:** repository-level tests for cursor correctness (forward,
  backward, around-anchor, tie-breaking on identical `created_at`,
  concurrent-insert-during-pagination correctness), service-level tests
  updated for the new signatures, HTTP integration tests
  (`server/tests/message.rs` or similar) covering `before_id`/`after_id`/
  `around_id`/`has_more_*` end-to-end, and a thread-replies-specific test
  replacing reliance on the old 1000-row hack.
- **Frontend:** hook tests for the live-tail vs anchored-history gating
  logic (this is the one place a subtle bug would be invisible without a
  test), scroll-anchoring math, and pill visibility logic.
- **Depends on:** all prior phases.
- **Checkpoint:** `cargo nextest run --workspace` and `pnpm test` (desktop)
  both green; this is also the gate before touching CLAUDE.md/docs.

### Phase 9 — Docs
- **Files:** `CLAUDE.md`, `server/README.md`/`desktop/README.md` if they
  describe message loading, `book/*.md` if it covers the message pane.
- **Tasks:** document the new pagination model and read-state tie-in per
  the existing "Update docs" step of the Implementation Loop.
- **Depends on:** Phase 8.

## Addendum — per-message unread indicator (added after initial planning)

Each unread message gets its own visual marker (dot/highlight) that clears
individually and immediately when that specific message is marked read —
not a one-time "new messages" divider line. This reuses the Phase 6/7
visibility-tracking and read-state-batching work directly, plus one small
addition to Phase 3:

- **Phase 3 addition:** `GET /channels/{id}/messages`,
  `GET /direct_messages/{id}/messages`, and the thread-replies endpoint each
  add a per-caller `is_unread: bool` to every message in the **HTTP response
  only** — computed via the existing `MessageReadRepository::unread_message_ids`
  batch lookup (already used internally by `search.rs`'s `is:unread`
  operator per ADR-015), same batch call as the attachments join. This does
  **not** go on the shared domain `Message` struct: that struct is also the
  payload for WebSocket broadcasts (`message.created`/`message.updated`) and
  MCP tool results, which are shared verbatim across every recipient in a
  conversation — baking in one caller's read state there would leak it to
  (or misrepresent it for) everyone else. `is_unread` is caller-relative and
  must stay confined to the per-request HTTP response DTO. Per ADR-015's
  existing rule, a caller's own authored messages are never unread.
- **Phase 5/6 addition:** the client keeps a local `Set<messageId>` of
  unread ids for the open conversation, seeded from each page's `is_unread`
  fields. Phase 7's visibility-triggered read-state batching optimistically
  removes an id from this set the instant that message is marked read
  (no waiting on a round trip), and incoming `read_state.updated` events
  (already broadcast to all of the same user's sessions per ADR-015) also
  remove ids, covering the cross-tab/cross-device case.
- **Phase 6 addition:** `MessageItem.tsx` renders a dot/marker when
  `message.id` is in the local unread set and the message isn't the
  caller's own.

## Execution order

Phase 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9, strictly sequential at the
phase level (each phase's checkpoint gates the next), though within Phase 6
the main-pane and thread-pane UI work can proceed in parallel once Phase 5
lands, since they touch different components.

## Next step

`/sc:implement` to execute this plan phase by phase, starting with Phase 0
(the ADR).
