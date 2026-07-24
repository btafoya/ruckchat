# Web UI Admin Panel — Requirements

## Status

Accepted — implemented in Phase 14.

## Goals

Provide a built-in administration experience in the Web UI (shared `desktop/src`)
so instance operators and organization managers can configure and moderate
RuckChat without using the CLI or raw API calls.

## Scope

Two admin surfaces:

- **Org-level admin** — for users who are owner or admin of the active organization.
- **Server-wide admin** — for users marked as server administrators.

## Server-Wide Administrator Model

- The **first registered user** automatically becomes a server administrator.
- Existing server administrators can promote or demote other users to/from server
  administrator.
- Demoting the **last remaining server administrator** is blocked.
- Server administrator status is stored on the user record (`users.is_server_admin`).

## Server-Wide Admin Capabilities

Server administrators can:

- **Organizations**: list, create, rename, archive, and delete all organizations;
  manage members in any organization.
- **Users**: list all users with pagination/filter, deactivate/reactivate accounts,
  reset passwords, update any user's display name and email.
- **Server settings**: configure global/instance-level settings such as default/max
  quotas, allowed signup domains, and maintenance mode.
- **Full org participation**: in any organization, post, edit, delete, join/leave
  channels, manage members, and moderate messages/files as if they were the org
  owner.
- **Impersonation**: act on behalf of any user; actions appear to come from the
  target user and are logged under both identities.
- **Audit log**: view the global audit log.

## Organization-Level Admin Capabilities

Organization owners and admins can manage:

- **Members**: invite users, remove members, change member roles.
- **Channels**: create, archive, update topic/purpose, manage membership.
- **Organization settings**: edit name/slug and configure file upload/storage quotas.
- **Custom roles**: create, list, edit, and delete.
- **Permissions**: create, list, edit, and delete.
- **Custom emoji**: create and list.
- **Teams**: create and list.

## Web UI Navigation Structure

- **Top-level server admin** under `/admin`:
  - `/admin/server/organizations`
  - `/admin/server/users`
  - `/admin/server/settings`
  - `/admin/server/audit-log`
  - `/admin/server/admins`
- **Org-level admin mode** reachable from the active organization context, scoped
  under `/org/:organizationId/admin/*`.
- **Route guards**: admin routes and navigation items are hidden from users without
  the required role. Direct navigation by unauthorized users returns 403.

## Audit Log

- A new `audit_log` table captures every admin action and relevant user security
  events (login, logout, password changes, message/file/channel deletions by
  admins, setting changes, promotions, impersonation sessions, etc.).
- Each entry records: timestamp, actor user ID, action type, target resource, target
  organization (if any), before/after snapshot or summary, impersonated user if
  applicable, and client IP where feasible.
- Full message content diffs are stored **only when an admin or impersonator edits
  a message**; normal user edits are metadata-only.
- Retention is **indefinite**.
- Only **server administrators** can view the audit log in the Web UI.
- Entries are **immutable** — no UI or API to delete or mutate them.

## Privacy & Disclosure

- Server administrators' ability to read any channel or direct message is **not
  visibly disclosed** in the end-user UI.

## Server Settings Source of Truth

- Operational server settings are stored in a new `server_settings` table and editable
  through the Web UI.
- `ruckchat.yaml` remains the hard instance config (database URL, bind address) and
  can explicitly override any soft setting, taking precedence at runtime.
- The server reads settings from the database by default, with YAML overrides
  layered on top.

## Non-Functional Requirements

- The admin UI reuses the existing shared `desktop/src` components and styling so
  it works for both desktop and Web UI builds.
- All new admin endpoints require appropriate authorization on the backend; UI
  gating is a convenience, not the security boundary.
- Server admin endpoints must enforce server administrator status; org admin
  endpoints must enforce org owner/admin or server administrator status.

## Open Questions Resolved

| # | Decision |
|---|----------|
| 1 | Org admins can fully edit custom roles and permissions, including name and key. |
| 2 | Server admins can fully impersonate users; actions appear from the user and are logged under both identities. |
| 3 | Audit log stores full message diffs only for admin/impersonator edits. |
| 4 | Server settings are database-backed and UI-editable, with `ruckchat.yaml` able to override any value. |

## Related

- `docs/DESIGN-Web-UI-Admin-Panel.md`
- `docs/ADR-010-Runtime-YAML-Configuration.md`
- `book/019-Web-UI.md`
