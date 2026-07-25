# ADR-017: Server-Stored Theme Preference

## Status

Accepted — implemented.

## Context

The UI theme system was originally client-side only: `desktop/src/hooks/useSettings.ts`
persisted `light` | `dark` | `system` in `localStorage`, and the active theme resolved to a CSS
class on `document.documentElement`. That worked for a single device, but switching devices or
browsers reset the choice, and the web client had to re-discover the user's preference on every
fresh load. Issue ISSUES17 asked for the theme preference to be stored server-side so it follows
the user across sessions and devices.

## Decision

- Extend the `users` table with a `theme TEXT NOT NULL DEFAULT 'system'` column.
- Add `theme` to the domain `User` aggregate (`crates/ruckchat-domain/src/user.rs`) and to all
  repository SELECT/INSERT/UPDATE paths (`server/src/repositories/user.rs`,
  `server/src/migrate.rs`, `crates/rocketchat2ruckchat/src/transform.rs`).
- Expose `theme` as an enum (`light`, `dark`, `system`) on `UserResponse` and accept it as an
  optional field on `UpdateProfileRequest` (`server/src/handlers/dto.rs`,
  `server/src/services/dto.rs`).
- Validate incoming theme values in `UserService::update_profile`; reject unknown values with
  `400 Bad Request`.
- The client-side settings store remains the source of truth for the *active* resolved theme
  (including `system` media-query resolution), but the server value is authoritative for the
  *user's saved preference*.
- On login, registration, and profile restoration, apply the server-returned theme to the local
  settings store (`desktop/src/hooks/useSession.ts`).
- The theme selector in `desktop/src/components/Settings.tsx` persists every change to the server
  via `PATCH /api/v1/users/me`.
- `serde(default = "theme_default")` on the domain `User` keeps existing serialized domain objects
  and any external consumers backward-compatible when deserializing a `User` without a `theme`
  field.

## Consequences

- The theme now follows the user across desktop and web clients.
- The domain `User` value object gains a new required field, so all direct constructions (migration
  tool, tests, snapshot import) must supply it.
- Adding the column requires a new migration, SQLx offline metadata refresh, and OpenAPI/schema
  regeneration.
- Invalid theme strings are rejected at the service layer rather than silently normalized, keeping
  the set of allowed values explicit and small.

## Migration

`migrations/migrations/20260725030000_users_theme.up.sql` adds the column with a `DEFAULT 'system'`
and back-fills existing rows. The matching `.down.sql` removes it.
