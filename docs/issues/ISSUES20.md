# ISSUES20 — Finish Direct Messages Modal

## Source

> Finish direct messages - clicking on + brings up a modal with no users listed - finish CRUD — open

## Research Summary

### Current state

- `StartDmModal.tsx` renders a checkbox list of organization members, filtered to exclude the current user (`desktop/src/components/StartDmModal.tsx:62-73`).
- It consumes `useOrgMemberContext().members`; if that context is empty, the modal shows "No other members in this organization."
- The `+` button is in `Sidebar.tsx` inside the "Direct messages" section.
- `useDirectMessages.ts` and `api.directMessages.ts` support starting and listing DM conversations.
- The reported symptom is that the modal opens with no users listed, suggesting either:
  - `useOrgMemberContext` is not populated in the sidebar context tree.
  - Members are filtered out incorrectly.
  - The modal does not trigger a member load.

### Gaps

1. **Diagnose empty member list** — determine whether the bug is missing context, missing data, or incorrect filtering.
2. **Member search/filter** — add a search input to the modal for organizations with many members.
3. **Multi-user DM creation** — the modal already supports multiple checkboxes; verify the API supports multi-member DMs.
4. **DM list CRUD** — after starting a DM, the sidebar should refresh and navigate to the new conversation.
5. **Hide/archive DM** — already implemented in `Sidebar.tsx` (`handleHideDm`), but may need confirmation.

### Affected files

- `desktop/src/components/StartDmModal.tsx` — add search, loading state, empty-state diagnosis.
- `desktop/src/components/Sidebar.tsx` — verify `+` button wiring and org member context availability.
- `desktop/src/hooks/useOrgMembers.ts` — ensure members load when the sidebar mounts.
- `desktop/src/hooks/useDirectMessages.ts` — refresh after starting a DM.
- `desktop/src/api/directMessages.ts` — verify start/list/hide endpoints.

## Open Questions

1. **Why is the user list empty?**
   - Needs reproduction: is `members` empty because the hook did not load, or because the organization has no other members?

2. **Should the modal support single-user and multi-user DMs equally?**
   - Yes, keep multi-select.
   - No, default to single-select and add an explicit "group" option.

3. **Should there be a confirmation before starting a DM that already exists?**
   - Yes, redirect to the existing conversation.
   - No, simply navigate to the existing one silently.

## Decisions

- Pending. First step is to reproduce the empty member list and confirm the data path.

## Status

Open.
