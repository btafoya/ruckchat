# ISSUES16 — Organization Home View Shows All Unread Messages

## Source

> The org view should show all unread messages for the logged in user linked to the actual channel/direct message view — open

## Research Summary

### Current state

- `ReadStateContext` provides unread counts per conversation via `useReadState` (`desktop/src/context/ReadStateContext.tsx`).
- `Sidebar.tsx` renders unread badges on channels and DMs using `useReadStateContext().counts` (`desktop/src/components/Sidebar.tsx:56`).
- `OrgIndexRoute` (the `/org` view) is currently an empty placeholder; there is no organization-level home page that lists unread messages.
- The server exposes `GET /organizations/{id}/unread_counts` (`docs/ADR-015-Search-And-Read-State.md`).

### Gaps

1. **No organization home page UI** — the `/org` route needs a real component that shows the active organization, recent/unread channels and DMs, and links into each conversation.
2. **No unread aggregation** — unread counts exist per conversation, but they are not summarized into an "Inbox" or "Unread" list.
3. **No conversation preview** — each row should show the channel/DM name, unread count, and possibly the latest message snippet.
4. **No empty state** — when everything is read, the view should show a helpful placeholder.

### Affected files

- `desktop/src/PlatformShell.tsx` — `OrgIndexRoute` currently renders `element={<OrgIndexRoute />}` and the component must be implemented/connected.
- `desktop/src/components/OrgIndex.tsx` — create the organization home view (does not exist yet).
- `desktop/src/hooks/useReadState.ts` — already exposes counts; may need message snippets.
- `desktop/src/hooks/useMessages.ts` / `desktop/src/api/messages.ts` — may need a recent-messages endpoint or use existing list endpoints.
- `desktop/src/components/Sidebar.tsx` — the org list link now has a meaningful destination.

## Open Questions

1. **What rows appear in the org home view?**
   - Only conversations with unread messages.
   - All conversations sorted by most recent activity, with unread badges.

2. **Should it be a true inbox across all organizations or per organization?**
   - Per organization (matches `/org` route).
   - Cross-organization global inbox.

3. **What data does each row show?**
   - Name, unread count, timestamp, last message preview, author avatar.

4. **Backend support needed?**
   - Use existing unread_counts + channel/DM list APIs.
   - Add a new `GET /organizations/{id}/inbox` endpoint returning recent unread items.

## Decisions

- Pending. Scope depends on whether the view is a simple unread list or a full recent-activity dashboard.

## Status

Open.
