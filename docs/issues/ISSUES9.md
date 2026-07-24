# ISSUES9 — Site Setting to Allow/Deny User Registrations

## Source

> Add to site setting allow registration checkbox option defaulting to on and add logic to allow/deny user registrations based on the setting — open

## Research Summary

### Current state

- `RegisterRequest` requires email, display_name, password, organization_name, and organization_slug (`server/openapi.yaml:3114-3121`).
- The first registered user becomes a server administrator (`docs/REQUIREMENTS-Web-UI-Admin-Panel.md`).
- `ServerSettings` currently contains `maintenance_mode_enabled`, `default_max_file_size_bytes`, `default_storage_quota_bytes`, and `allowed_signup_domains` (`server/openapi.yaml:3476-3483`).
- `UpdateServerSettingsRequest` mirrors those fields (`server/openapi.yaml:3484-3491`).
- There is no `allow_registration` field.
- The registration handler likely always creates a new user and organization; it does not check a registration gate.

### Gaps

1. **Schema addition** — add `allow_registration: boolean` to `ServerSettings` and `UpdateServerSettingsRequest` with default `true`.
2. **Database migration** — add a column to the `server_settings` table with default `true`.
3. **Backend enforcement** — reject `POST /api/v1/auth/register` with `403 Forbidden` when `allow_registration` is false.
4. **UI checkbox** — add an "Allow new user registrations" checkbox to `ServerAdminSettings.tsx`.
5. **Auth screen handling** — when registration is disabled, hide or disable the register form/tab and show an explanatory message.
6. **YAML override** — ensure the setting can be overridden in `ruckchat.yaml` per the existing override precedence.

### Affected files

- `server/openapi.yaml` — add `allow_registration` to server settings schemas.
- `migrations/migrations/` — new migration for `server_settings.allow_registration`.
- `server/src/services/server_settings.rs` — load and merge the new setting.
- `server/src/services/auth.rs` or `server/src/handlers/auth.rs` — gate registration.
- `server/src/config.rs` — add optional YAML override field.
- `desktop/src/components/admin/ServerAdminSettings.tsx` — add checkbox.
- `desktop/src/components/AuthScreen.tsx` — hide/disable registration when disabled.

## Open Questions

1. **What is the default behavior?**
   - `true` (registration allowed) by default, matching the issue request.
   - `false` (registration disabled) for secure-by-default deployments.

2. **When registration is disabled, can server admins still create users?**
   - Yes, server admin user creation remains available.
   - No, all new accounts are blocked including admin creation.

3. **Should the setting also affect invitations to existing organizations?**
   - Block only public self-registration; org invites still work.
   - Block all new account creation, including invite acceptance by non-existing users.

4. **Where should the gate be enforced?**
   - In the REST handler (`auth.rs`) before calling the auth service.
   - Inside the auth service so MCP/plugin routes also respect it.

## Decisions

- Registration gate behavior: when disabled, block public self-registration only; server-admin user creation and organization invites still work.
- Enforcement layer: REST handler before the auth service.
