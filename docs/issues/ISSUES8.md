# ISSUES8 — User Editor Modal in Server Admin

## Source

> The user editor is just a line editor in the user list when it should be a modal with the ability to manage all aspects of the user account — open

## Research Summary

### Current state

- `ServerAdminUsers.tsx` renders the server-wide user list.
- The editor is described as a "line editor in the user list" (inline editing), which limits the fields that can be exposed.
- The backend `UpdateServerUserRequest` supports `display_name`, `avatar_url`, and `email` (`server/openapi.yaml:3467-3472`).
- Server admins can also deactivate/reactivate accounts, reset passwords, and toggle `is_server_admin` (per `docs/REQUIREMENTS-Web-UI-Admin-Panel.md`).
- The current inline UI likely cannot expose all these actions cleanly.

### Gaps

1. **Modal editor** — open a full-screen or centered modal when editing a user from the list.
2. **Editable profile fields** — display name, email, avatar URL.
3. **Security actions** — reset password (with generated password shown), deactivate/reactivate account.
4. **Admin promotion/demotion** — toggle `is_server_admin` with guard against demoting the last admin.
5. **Audit context** — show recent audit log entries for the user (optional but useful).
6. **Validation and confirmation** — confirm destructive actions (deactivate, delete if supported).

### Affected files

- `desktop/src/components/admin/ServerAdminUsers.tsx` — replace inline editing with a modal trigger.
- `desktop/src/components/admin/EditUserModal.tsx` (new) — modal form and action buttons.
- `desktop/src/api/serverAdmin.ts` — verify all user update/admin endpoints are wrapped.
- `server/src/handlers/server_admin.rs` and `server/src/services/server_admin.rs` — ensure password reset, deactivation, and admin toggling are exposed.

## Open Questions

1. **Which user fields must be editable in the modal?**
   - Display name, email, avatar URL only.
   - Plus password reset and server-admin toggle.
   - Plus deactivate/reactivate toggle.

2. **Should deactivation be a separate destructive confirmation, or a simple toggle?**
   - Simple toggle with a confirmation dialog.
   - A dedicated "Danger zone" section at the bottom of the modal.

3. **Should the modal support creating a new user, or only editing existing users?**
   - Editing only; keep existing create-user flow.
   - Combine create and edit in the same modal component.

4. **Should users be permanently deletable from the server admin UI?**
   - No, only deactivate (safer, preserves history).
   - Yes, with a strong confirmation and audit log entry.

## Decisions

- User editor modal actions: profile fields, server-admin toggle, password reset, and user deletion (with confirmation and audit logging).
- Destructive actions: presented in a "Danger zone" section with confirmation dialogs.
- Modal scope: support both creating new users and editing existing users in the same modal.
