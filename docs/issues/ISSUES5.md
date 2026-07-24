# ISSUES5 — Complete Direct Message Functionality

## Source

> Direct messages is missing all functionality. Fully complete the UI allowing users to message others in their organization in the same fashion slack does — open

## Research Summary

### Current state

- Backend models exist for `DirectMessageConversation` with `member_ids` (`server/openapi.yaml:3081-3090`).
- `StartDmRequest` exists (`server/openapi.yaml:3171-3174`) and the `POST` endpoint likely creates a DM conversation.
- `desktop/src/hooks/useDirectMessages.ts` loads DM conversations for an organization.
- Routes exist for `/org/:organizationId/dm/:dmId` (`desktop/src/PlatformShell.tsx:201`).
- The composer already accepts `conversationType: 'direct_message'` (`Composer.tsx:10-12`) and `useDirectMessageContext` is consumed for mention candidates (`Composer.tsx:30`, `60-70`).
- The sidebar DM section and start-DM UI are the likely missing pieces (exact state depends on `Sidebar.tsx` and `Shell.tsx`, currently being reviewed).

### Gaps

1. **Start a DM** — UI to pick an organization member and start a conversation. Should allow multi-user DMs or only 1:1?
2. **DM list in sidebar** — render DM conversations with member display names, avatars, and unread indicators.
3. **DM threads** — the route supports `/dm/:dmId/thread/:messageId`, but the thread pane may not be wired for DMs.
4. **DM composer** — verify the composer works when `conversationType === 'direct_message'` and messages are routed to the DM endpoint.
5. **Presence / typing** — typing indicators and presence should work in DMs.
6. **Notifications** — DM notifications should already be wired via `useNotifications`, but verify behavior.

### Affected files

- `desktop/src/components/Sidebar.tsx` — add DM section and "New message" button.
- `desktop/src/components/StartDmModal.tsx` or inline search (new) — member picker.
- `desktop/src/hooks/useDirectMessages.ts` — verify load/start/reload logic.
- `desktop/src/api/directMessages.ts` — verify start and list endpoints.
- `desktop/src/components/MessagePane.tsx` — handle DM conversation type.

## Open Questions

1. **Are DMs limited to two users or multi-user (group DMs)?**
   - 1:1 only for MVP.
   - Multi-user group DMs allowed because the schema supports `member_ids`.

2. **How are DMs identified in the sidebar?**
   - By the other member's display name.
   - By a combined title built from member names.

3. **Can users leave or delete a DM conversation?**
   - No, DMs are permanent (Slack-style).
   - Users can hide/archive a DM from the sidebar.

4. **Should starting a DM reuse the existing channel-like composer, or have a dedicated inline experience?**
   - A modal that searches members and creates the DM.
   - A dedicated "New message" route with a member search pane.

## Decisions

- DM scope: support multi-user group DMs (schema already permits `member_ids[]`).
- DM list naming: combined member display names (e.g., "Alice, Bob, Carol").
- DM lifecycle: users can hide/archive a DM from their own sidebar; the conversation remains for other members.
- Start DM UX: member-search modal launched from the sidebar.
