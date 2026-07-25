# ADR-015: Global Search and Server-Side Read State

## Status

Accepted — implemented.

## Context

Two features were requested: editing a previously sent message, and a global
search across messages, channels, people, and files. Investigation before
implementation found that message editing was already fully wired end to end
on the backend (`Message::edit`, `MessageService::edit_message`, `PATCH
/messages/{message_id}`, and the `message.updated` WebSocket event) and even
partially on the desktop client (`useRealtimeStore` already dispatched
`message.updated` to `updateMessage`); only the API client call and the
Composer UI were missing.

Message full-text search also already existed (`messages.content_tsv`
generated column, `MessageRepository::search`) but was reachable only through
the MCP `search_messages` tool, not a REST endpoint for the desktop/web UI.
Extending search to channels, people, and files, and adding Gmail-style query
operators (`from:`, `in:`, `has:attachment`, `before:`/`after:`, `is:unread`),
required new design decisions. The `is:unread` operator in turn required
promoting unread tracking from the desktop client's `localStorage`-only model
(`useUnread.ts`) to a server-side, per-user, per-message read state, since
`is:unread` needs to be evaluated server-side and consistently across a
user's devices.

## Decision

- **Edit message**: no backend changes. `desktop/src/api/messages.ts` adds
  `MessagesApi.edit`; `Composer.tsx` gains an edit mode (loads the target
  message's content, relabels the submit button "Save", adds "Cancel") driven
  by `editingMessage`/`startEdit`/`cancelEdit`/`saveEdit` added to
  `useMessages`. `MessageItem.tsx` shows an "Edit" action only for the
  caller's own, non-deleted, non-pending messages. No time limit, no edit
  history — only an `(edited)` indicator via the existing `updated_at`.
- **Search matching strategy is mixed, not uniformly full-text**: messages
  keep the existing Postgres `tsvector`/GIN full-text search. Channels,
  people, and files reuse the "load the visible/organization set, then filter
  in-memory with a lowercase `.contains()`" pattern already established by
  `OrganizationService::search_members` for @mention autocomplete, rather than
  adding new generated tsvector columns to those tables. Those tables are
  small per-organization and don't need stemming or ranking; matching the
  existing pattern avoids new schema surface for no measurable benefit at this
  scale.
- **Operator parsing happens server-side** on the raw query string
  (`server/src/services/search.rs::parse_query`), not client-side. The client
  sends the search box text verbatim. Operators are applied as an in-memory
  post-filter over the FTS-returned message page (capped at 100 rows), which
  is simpler than rewriting the SQL per operator combination.
- **`SearchService` composes existing services** (`MessageService`,
  `ChannelService`, `OrganizationService`, `FileService`) rather than
  reimplementing authorization or visibility checks; e.g. channel search
  reuses `ChannelService::list_channels_in_organization`, which already
  applies public/private visibility.
- **Read state is a new first-class server concept**: a `message_reads(user_id,
  message_id, read_at)` table, one row per (user, message) once read — the
  simplest, most exact model, accepted for the write-volume cost in busy
  channels rather than a cheaper high-water-mark scheme. It applies uniformly
  to channels and DMs. It is **internal only**: no "seen by"/read-receipt UI
  is exposed; the data exists solely to compute unread badges and back
  `is:unread`.
- **Hard cutover, no dual-read period**: `desktop/src/hooks/useUnread.ts` and
  its `ruckchat_unread_counts` localStorage key are deleted outright in the
  same change that ships `useReadState.ts`. Existing local unread counts are
  discarded at upgrade; there is no migration path for them.
- **Cross-device sync** reuses the existing single-user WebSocket broadcast
  mechanism (`ConnectionManager::broadcast_to_users` with a one-element
  slice, the same pattern `publish_mention` already uses) via a new
  `read_state.updated` event, rather than introducing a new transport.
- **Own messages are never "unread"**: both `unread_counts_by_conversation`
  and `unread_message_ids` exclude the caller's own authored messages, so a
  user's own posts never appear in their own badges or `is:unread` search
  results.
- A new `Sidebar`/`AuthenticatedShell` split was introduced via
  `ReadStateContext` so both consumers of unread counts share one
  `useReadState` instance instead of two independent, potentially
  inconsistent hook instances (the old `useUnread` was called separately in
  both places).

## Consequences

### Positive

- Message editing shipped as a small, low-risk client-only change because the
  backend and real-time plumbing were already correct and tested.
- Search reuses proven authorization paths from existing services instead of
  duplicating visibility logic, reducing the risk of a search-specific
  privacy leak.
- Read state becomes consistent across devices/sessions for the first time,
  fixing a pre-existing `localStorage` limitation as a side effect.

### Negative

- One `message_reads` row per (user, message) means write volume scales with
  `members × messages` in busy public channels. No batching/debouncing was
  added in this iteration; if this becomes a bottleneck, the fix is changing
  the storage model (e.g. a high-water-mark plus an exceptions table), not the
  API surface.
- Search's operator filters run in-memory after the FTS query, so a query
  combining `is:unread`/`has:attachment`/`in:`/`from:` with a very common free
  text term could still discard most of a 100-row page; there is no operator
  pushdown into the SQL query in this iteration.
- Channels/people/files search is `O(n)` over the organization's full list of
  each type; acceptable at current scale, but would need real pagination or
  indexed search if any of those tables grow large.

## Implementation

- `migrations/migrations/20260725020000_message_reads.up.sql`/`.down.sql`
- `crates/ruckchat-domain/src/repositories.rs` —
  `MessageReadRepository`; `FileRepository::message_ids_with_attachments`.
- `server/src/repositories/message_read.rs`,
  `server/src/repositories/file.rs` — SQLx implementations.
- `server/src/services/read_state.rs` — `ReadStateService`.
- `server/src/services/search.rs` — `parse_query`, `SearchService`.
- `server/src/services/events.rs`, `server/src/websocket/bus.rs`,
  `server/src/plugins/bus.rs`, `server/src/services/web_push.rs` —
  `publish_read_state_updated` / `ServerEvent::ReadStateUpdated`.
- `server/src/handlers/channel.rs`, `server/src/handlers/direct_message.rs` —
  `mark_read`; `server/src/handlers/organization.rs` — `unread_counts`;
  `server/src/handlers/search.rs` — `search`.
- `server/src/handlers/dto.rs` — `SearchResponse` (maps people to
  `UserResponse` so password hashes never serialize), `UnreadCountsResponse`.
- `server/src/state.rs` — `AppState.read_state`, `AppState.search`.
- `server/openapi.yaml` — `MarkReadRequest`, `UnreadCountsResponse`,
  `SearchResponse` schemas and their paths.
- `server/tests/search_and_read_state.rs` — integration tests.
- `desktop/src/api/messages.ts`, `desktop/src/api/search.ts` — new API
  clients; `desktop/src/api/channels.ts`, `desktop/src/api/directMessages.ts`,
  `desktop/src/api/organizations.ts` — `markRead`/`unreadCounts` additions.
- `desktop/src/hooks/useReadState.ts` — replaces `desktop/src/hooks/useUnread.ts`
  (deleted).
- `desktop/src/context/ReadStateContext.tsx` — shared read-state instance.
- `desktop/src/hooks/useMessages.ts` — `editingMessage`/`startEdit`/
  `cancelEdit`/`saveEdit`.
- `desktop/src/components/Composer.tsx`, `desktop/src/components/MessageItem.tsx`
  — edit-mode UI.
- `desktop/src/components/SearchResultsPage.tsx`, `desktop/src/components/Shell.tsx`
  — search results route and persistent top-bar search input.
- `desktop/src/PlatformShell.tsx` — wiring for `useReadState`, mark-read
  effect, `ReadStateProvider`, and the `/org/:organizationId/search` route.

## Related

- `docs/ADR-006-WebSocket-Real-Time-Events.md`
- `docs/ADR-007-MCP-Server.md` (existing `search_messages` MCP tool)
