# ISSUES7 — Complete Organization Admin UI

## Source

> Organization Admin UI is incomplete — open

## Research Summary

### Current state

- Org admin routes are declared in `PlatformShell.tsx`: settings, members, roles, permissions, emoji, teams (`desktop/src/PlatformShell.tsx:183-191`).
- `OrgAdminSettings.tsx` is implemented with file-size and storage-quota inputs.
- `OrgAdminMembers`, `OrgAdminRoles`, `OrgAdminPermissions`, `OrgAdminEmoji`, and `OrgAdminTeams` exist as route targets; their implementation depth is still being verified.
- The backend has endpoints for roles, permissions, emoji, and teams (schemas exist in `server/openapi.yaml`).

### Gaps

1. **Members management** — invite by email, remove members, change roles.
2. **Roles** — create, list, edit, delete custom roles and assign permissions.
3. **Permissions** — create/list custom permission keys.
4. **Custom emoji** — upload/list/delete custom emoji.
5. **Teams** — create/list/delete teams, add/remove members, assign team rooms.
6. **Organization profile** — edit organization name/slug (settings may cover name only; slug editing is not confirmed).
7. **Consistency** — ensure all org admin screens share a common layout, loading, and error handling pattern.

### Affected files

- `desktop/src/components/admin/OrgAdminMembers.tsx`
- `desktop/src/components/admin/OrgAdminRoles.tsx`
- `desktop/src/components/admin/OrgAdminPermissions.tsx`
- `desktop/src/components/admin/OrgAdminEmoji.tsx`
- `desktop/src/components/admin/OrgAdminTeams.tsx`
- `desktop/src/components/admin/OrgAdminShell.tsx`
- `desktop/src/api/orgAdmin.ts`
- `server/src/handlers/org_admin.rs` and `server/src/services/organization.rs`

## Open Questions

1. **Which org admin screens are the highest priority?**
   - Members + settings only for MVP.
   - Members, roles, and permissions first.
   - All screens (members, roles, permissions, emoji, teams) before marking complete.

2. **Should custom roles/permissions/emoji/teams be editable from the same shell, or moved to separate future phases?**
   - Include them now because the backend already supports them.
   - Defer teams and emoji to keep the UI focused.

3. **Should the org admin sidebar be collapsible or tab-based on small screens?**
   - Collapsible sidebar like the main shell.
   - Horizontal tabs under the org admin header.

## Decisions

- Scope: implement all org admin screens (members, roles, permissions, emoji, teams) before marking the issue complete.
- Mobile layout: collapsible sidebar like the main shell.
