# Web UI Admin Panel — Design

## Status

Accepted — implemented in Phase 14.

## Overview

This design adds a multi-layer administration system to RuckChat:

1. **Backend authorization + data model** — server admin flag, audit log table, server settings table.
2. **REST API additions** — server admin endpoints, org admin endpoints, audit log endpoints, impersonation endpoints.
3. **Service-layer additions** — server admin service, audit service, settings service, impersonation guard.
4. **Web UI components/routes** — admin shell, server admin pages, org admin pages, shared admin tables/forms.

The design follows existing patterns: Axum REST, SQLx repositories, domain crate
entities, shared `desktop/src` React components, OpenAPI-first schema generation.

## Data Model

### `users.is_server_admin`

```sql
ALTER TABLE users ADD COLUMN is_server_admin BOOLEAN NOT NULL DEFAULT FALSE;
```

- The first user created is set to `TRUE`.
- Only server admins can update this flag on other users.

### `server_settings`

```sql
CREATE TABLE server_settings (
    key VARCHAR(128) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id)
);
```

Seed keys:

- `maintenance_mode_enabled` (`boolean`)
- `default_max_file_size_bytes` (`i64`)
- `default_storage_quota_bytes` (`i64`)
- `allowed_signup_domains` (`text[]` or JSON)

Runtime precedence:

1. `ruckchat.yaml` override if key present.
2. `server_settings` table value.
3. Hard-coded default.

### `audit_log`

```sql
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id UUID NOT NULL REFERENCES users(id),
    impersonated_user_id UUID REFERENCES users(id),
    organization_id UUID REFERENCES organizations(id),
    action VARCHAR(64) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id UUID,
    metadata JSONB,
    ip_address INET
);
```

- `metadata` holds before/after snapshots, diffs, or action-specific context.
- Append-only; no updates or deletes.
- Indexed on `actor_id`, `organization_id`, `occurred_at`, and `action`.

## Authorization

- Keep existing `owner | admin | member` organization role enum.
- Add **server admin** as a separate cross-cutting flag on `User`, not a role within an org.
- Update `AuthorizationService`:
  - If caller `is_server_admin`, allow any org action without membership.
  - Add `require_server_admin(caller_id)` guard returning `Forbidden` if false.

## Impersonation

- Server admin starts an impersonation session by selecting a target user.
- The server issues a short-lived **impersonation session claim** (e.g., an additional
cookie claim `impersonating: target_user_id`).
- All subsequent actions are authorized as the target user but audit-logged with both
`actor_id` (server admin) and `impersonated_user_id`.
- The UI shows a persistent **“Acting as Alice” banner** with an exit button.

## REST API

### Server admin routes

| Method | Route | Description |
|--------|-------|-------------|
| GET | `/api/v1/server/organizations` | List all organizations |
| POST | `/api/v1/server/organizations` | Create organization |
| PATCH | `/api/v1/server/organizations/{id}` | Rename organization |
| DELETE | `/api/v1/server/organizations/{id}` | Delete organization |
| GET | `/api/v1/server/users` | List all users with pagination/filter |
| GET | `/api/v1/server/users/{id}` | Get user details |
| PATCH | `/api/v1/server/users/{id}` | Update user |
| POST | `/api/v1/server/users/{id}/reset-password` | Reset user password |
| POST | `/api/v1/server/users/{id}/promote` | Promote to server admin |
| POST | `/api/v1/server/users/{id}/demote` | Demote from server admin |
| GET | `/api/v1/server/settings` | Read server settings |
| PUT | `/api/v1/server/settings` | Update server settings |
| GET | `/api/v1/server/audit-log` | Query audit log with filters |
| POST | `/api/v1/server/impersonate` | Start impersonation |
| DELETE | `/api/v1/server/impersonate` | End impersonation |

### Org admin additions

| Method | Route | Description |
|--------|-------|-------------|
| GET | `/api/v1/organizations/{id}/members` | List members |
| GET | `/api/v1/admin/organizations/{id}/settings` | Read org settings |
| PUT | `/api/v1/admin/organizations/{id}/settings` | Update org settings |
| PATCH | `/api/v1/admin/organizations/{id}/roles/{role_id}` | Edit custom role |
| DELETE | `/api/v1/admin/organizations/{id}/roles/{role_id}` | Delete custom role |
| PATCH | `/api/v1/admin/organizations/{id}/permissions/{permission_id}` | Edit permission |
| DELETE | `/api/v1/admin/organizations/{id}/permissions/{permission_id}` | Delete permission |
| DELETE | `/api/v1/admin/organizations/{id}/emoji/{emoji_id}` | Delete custom emoji |
| DELETE | `/api/v1/admin/organizations/{id}/teams/{team_id}` | Delete team |

### Audit log query

- Query params: `from`, `to`, `actor_id`, `organization_id`, `action`, `resource_type`,
  `limit`, `offset`.
- Response: list of audit entries ordered by `occurred_at DESC`.

## Service Layer

### `ServerAdminService`

Dependencies:

- `UserRepository`
- `OrganizationRepository`
- `OrganizationMembershipRepository`
- `OrganizationSettingsRepository`
- `AuditService`
- Password reset helper

Responsibilities:

- CRUD users and organizations.
- Promote/demote server admins with last-admin lockout.
- Manage server settings.
- Start/stop impersonation sessions.

### `AuditService`

- Append-only writer used by all services.
- No business logic; just persists events.
- Exposed via repository trait for testability.

### `ServerSettingsService`

- Reads from `server_settings` table.
- Applies `ruckchat.yaml` overrides.
- Returns merged settings.

### Updated `AdminService`

- Add endpoints for edit/delete role, edit/delete permission, delete emoji, delete team.
- Add org settings read/update.
- Reuse existing authorization.

### Updated `AuthorizationService`

- Add `is_server_admin(caller_id)` check.
- Modify org action checks: if server admin, bypass membership/role checks.

## Web UI

### Routes

In `desktop/src/PlatformShell.tsx`:

```tsx
<Route path="/admin/server/*" element={<ServerAdminShell />} />
<Route path="/org/:organizationId/admin/*" element={<OrgAdminShell />} />
```

### New components

- `ServerAdminShell.tsx` — layout with tabs for Organizations, Users, Settings, Audit Log, Server Admins.
- `ServerAdminOrganizations.tsx` — table + create/edit/delete forms.
- `ServerAdminUsers.tsx` — user table with promote/demote/impersonate/deactivate actions.
- `ServerAdminSettings.tsx` — form for server settings with YAML-override notice.
- `ServerAdminAuditLog.tsx` — filterable table of audit entries.
- `ServerAdminAdmins.tsx` — list of server admins with add/remove.
- `OrgAdminShell.tsx` — context-aware org admin layout.
- `OrgAdminMembers.tsx`, `OrgAdminChannels.tsx`, `OrgAdminSettings.tsx`, `OrgAdminRoles.tsx`,
  `OrgAdminPermissions.tsx`, `OrgAdminEmoji.tsx`, `OrgAdminTeams.tsx`.

### API client modules

- `desktop/src/api/serverAdmin.ts`
- `desktop/src/api/orgAdmin.ts`
- Update `desktop/src/api/types.ts` from regenerated OpenAPI schema.

### Navigation

- Add **“Administration”** item in `Sidebar.tsx` for org admin context when the user is owner/admin.
- Add a top-level **“Server Admin”** menu or avatar dropdown for server admins.

### Impersonation UI

- A persistent banner: **“You are acting as Alice. Exit impersonation.”**
- Shown globally when `impersonated_user_id` is present in session/token.

## Sequence Diagrams

### Promote user to server admin

```text
Web UI → POST /api/v1/server/users/{id}/promote
         AuthUser extractor → session cookie
         handler → ServerAdminService::promote_user
         service → require_server_admin(caller)
                 → update user.is_server_admin = true
                 → AuditService::record(PROMOTE_SERVER_ADMIN)
         response → 200
```

### Impersonate and edit a message

```text
Web UI → POST /api/v1/server/impersonate { target_user_id }
         service → require_server_admin
                 → issue impersonation session claim

Web UI → PATCH /messages/{id} (with claim)
         handler → auth sees impersonated_user_id as effective user
                 → message service edits as target user
                 → AuditService::record(MESSAGE_EDIT, actor=admin,
                                       impersonated=target, diff=...)
```

## OpenAPI / Schema

- Add all new endpoints to `server/openapi.yaml`.
- Add schemas:
  - `ServerUser`
  - `ServerOrganization`
  - `ServerSettings`
  - `AuditLogEntry`
  - `ImpersonateRequest`
  - `UpdateServerSettingsRequest`
  - `UpdateRoleRequest`, `UpdatePermissionRequest`
- Regenerate `desktop/src/api/schema.ts`.

## Testing Strategy

- **Unit tests**: service-layer promotion/demotion lockout, settings merge, authorization server-admin bypass.
- **Integration tests**: new REST endpoints with auth, audit log writes, impersonation flow.
- **Web UI tests**: route guards, admin menu visibility, impersonation banner render.

## Migrations & Packaging

Add SQLx migrations:

1. `users.is_server_admin`
2. `server_settings` table
3. `audit_log` table

Also update:

- `server/openapi.yaml`
- `docs/ADR-*.md` if architecture changes meaningfully
- Regenerate SQLx offline query metadata

## Implementation Order

1. Database migrations (`users.is_server_admin`, `server_settings`, `audit_log`).
2. Domain/repository traits and SQLx implementations.
3. Authorization updates for server admin bypass.
4. `ServerAdminService`, `AuditService`, `ServerSettingsService`.
5. Handlers and OpenAPI updates.
6. Integration tests for backend.
7. Web UI API client and types.
8. Web UI admin components and routes.
9. Web UI tests.
10. Regenerate `.sqlx/`, run full implementation loop, commit.

## Related

- `docs/REQUIREMENTS-Web-UI-Admin-Panel.md`
- `docs/ADR-010-Runtime-YAML-Configuration.md`
- `book/019-Web-UI.md`
