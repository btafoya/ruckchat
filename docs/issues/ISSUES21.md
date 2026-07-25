# ISSUES21 — Finish Server Admins View CRUD

## Source

> Finish CRUD for `Server Admins View` list (Completely missing) — open

## Research Summary

### Current state

- `ServerAdminAdmins.tsx` exists and implements a list of server administrators plus an "Add Admin" promote flow (`desktop/src/components/admin/ServerAdminAdmins.tsx`).
- The component loads admins via `api.serverAdmin.listServerAdmins(token)` and all users via `api.serverAdmin.listUsers(token)`.
- It supports promoting an active user to server admin but does not support:
  - Demoting a server admin back to a regular user.
  - Searching/filtering the admin list.
  - Showing audit/context for admin grants.
  - Removing the user from the list view if they are the last admin (already guarded server-side).
- The tab exists in `ServerAdminShell.tsx` at `/admin/server/admins` (`desktop/src/components/admin/ServerAdminShell.tsx:8`).
- Backend support: `server/src/handlers/server_admin.rs` has promote/demote endpoints and a list server admins endpoint (per Phase 14).

### Gaps

1. **Missing demote action** — the UI has no way to remove server-admin privileges from a listed admin.
2. **No confirmation for destructive actions** — promote/demote should require confirmation, especially demoting the last server admin, which the server rejects.
3. **No search/filter** — the admin list could grow large; add a search by display name/email.
4. **No empty/additional states** — loading, error, and "no admins" states are minimal.
5. **List vs. candidate UX** — the current promote panel mixes search candidates with the admin list; consider separating concerns.

### Affected files

- `desktop/src/components/admin/ServerAdminAdmins.tsx` — add demote, search, confirmation, and improved empty/error states.
- `desktop/src/api/serverAdmin.ts` — verify `demoteUser` / `promoteUser` methods exist.
- `desktop/src/components/admin/ServerAdminShell.tsx` — no structural changes unless the tab icon changes (see ISSUES14).
- `server/src/handlers/server_admin.rs` — verify demote endpoint and error handling.
- `server/src/services/server_admin.rs` — verify last-admin guard.

## Open Questions

1. **Demote UX**
   - Show a "Demote" button on each admin row.
   - Move admin management into the existing `EditUserModal` (see ISSUES8) instead of a dedicated admins screen.

2. **Self-demotion guard**
   - Prevent a server admin from demoting themselves without another admin present.
   - Allow it but warn that they will lose access.

3. **Audit trail**
   - Record promote/demote actions in the server audit log (already exists; verify it is written).

## Decisions

- Pending. Depends on whether admin management should stay in a dedicated screen or move to `EditUserModal`.

## Status

Open.
