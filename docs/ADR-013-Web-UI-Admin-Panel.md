# ADR-013: Web UI Admin Panel

## Status

Accepted — implemented in Phase 14.

## Context

RuckChat instances need to be configured and moderated without CLI access.
Instance operators must manage users, organizations, and global settings, while
organization owners need self-service control over their own organizations.
We needed to decide where the authorization model lives, which settings are
server-wide vs. per-organization, how to keep an immutable record of admin
actions, and how the Web UI (shared with the desktop client) should expose these
capabilities.

## Decision

We will add a two-tier administration system exposed through the existing
shared React code in `desktop/src`.

- **Server-wide admin flag**: `users.is_server_admin` marks cross-cutting server
  administrators. The first registered user is automatically promoted.
- **Server admin bypass**: A server administrator may perform any organization
  action (edit channels, manage members, moderate messages) without being a
  member of that organization. Existing org-level checks short-circuit when the
  caller is a server admin.
- **Database-backed server settings**: A new `server_settings` table stores
  soft global settings (maintenance mode, quotas, allowed signup domains).
  `ruckchat.yaml` remains the hard instance config and can override any soft
  setting, taking precedence at runtime.
- **Append-only audit log**: Every admin action and security-relevant event is
  written to an `audit_log` table with actor, target resource, organization,
  JSONB metadata, and IP address. Entries are immutable and never deleted.
- **Impersonation**: Server administrators can act on behalf of any user. The
  target user's identity is used for authorization, while the real actor is
  preserved in the session and audit log. UI will show an impersonation banner.
- **REST surface**:
  - Server admin endpoints live under `/api/v1/server/*`.
  - Org-level admin endpoints live under `/api/v1/admin/organizations/{id}/*`.
- **Web UI routes**:
  - `/admin/server/*` for the server-wide admin shell.
  - `/org/:organizationId/admin/*` for organization-level administration of the
    active organization.
- **Navigation gating**: `Sidebar.tsx` shows a "Server Admin" link for server
  admins and an "Admin" link when the user is the organization owner or a
  server admin. Direct navigation to admin pages by unauthorized users is
  blocked by backend authorization returning 403.
- **Schema and client**: All endpoints are documented in
  `server/openapi.yaml`; `desktop/src/api/schema.ts` is regenerated. New
  `serverAdmin.ts` and `orgAdmin.ts` API modules wrap the endpoints.

## Consequences

### Positive

- Instance operators get a first-class browser admin experience without
  additional tooling.
- The server admin bypass keeps org-level code paths unchanged; authorization
  checks simply return `true` for server admins.
- Audit log provides a durable record for compliance and troubleshooting.
- Database-backed settings allow runtime changes without server restarts, while
  YAML overrides preserve operator-level control.
- Shared React components keep desktop and Web UI admin surfaces identical.

### Negative

- Server administrators can read any channel or DM; this is intentionally not
  visibly disclosed in the user UI.
- The audit log is never pruned; operators must manage disk growth externally.
- Impersonation increases the blast radius of a compromised server admin
  account.
- Adding `is_server_admin` to `UserResponse` leaks the flag to all clients.
  This is accepted because it is needed for UI gating and is not considered
  secret metadata.

## Implementation

- `migrations/migrations/20260724000000_users_is_server_admin.{up,down}.sql`
- `migrations/migrations/20260724000001_server_settings.{up,down}.sql`
- `migrations/migrations/20260724000002_audit_log.{up,down}.sql`
- `server/src/services/server_admin.rs` — user, organization, admin, settings,
  and impersonation operations.
- `server/src/services/audit.rs` — append-only audit writer.
- `server/src/services/server_settings.rs` — database settings with YAML
  override merge.
- `server/src/handlers/server_admin.rs` — server admin and impersonation
  handlers.
- `server/src/handlers/admin.rs` — org admin additions.
- `server/tests/server_admin.rs` — integration tests for the admin REST
  surface.
- `server/openapi.yaml` — documented admin endpoints and schemas.
- `desktop/src/api/serverAdmin.ts` and `desktop/src/api/orgAdmin.ts` — admin
  API clients.
- `desktop/src/components/admin/*.tsx` — admin shells, tables, and forms.
- `desktop/src/PlatformShell.tsx` — added `/admin/server/*` and
  `/org/:organizationId/admin/*` route trees.
- `desktop/src/components/Sidebar.tsx` — admin navigation links gated on role.

## Related

- `docs/REQUIREMENTS-Web-UI-Admin-Panel.md`
- `docs/DESIGN-Web-UI-Admin-Panel.md`
- `book/019-Web-UI.md`
- `docs/ADR-010-Runtime-YAML-Configuration.md`
