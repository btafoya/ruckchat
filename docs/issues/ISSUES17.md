# ISSUES17 — Theme Preference Stored in Server User Profile

## Source

> The preferred theme doesn't survive logging out and returning; this should be a profile setting stored in the server user profile — open

## Research Summary

### Current state

- `useSettings.ts` stores `theme` (`light` | `dark` | `system`) in `localStorage` under `ruckchat_settings` (`desktop/src/hooks/useSettings.ts:4-12`).
- `Settings.tsx` provides a theme toggle with Light/Dark/System buttons (`desktop/src/components/Settings.tsx:7-11`, `75-93`).
- The theme is resolved client-side via `matchMedia('(prefers-color-scheme: dark)')` (`desktop/src/hooks/useSettings.ts:23-31`).
- The server user profile (e.g., `User` entity) does not currently have a `theme` field.
- `AuthScreen.tsx` redirects to `/` after login without applying any server-stored theme.

### Gaps

1. **No server-side theme field** — need to add `theme` to the user model, database, and API.
2. **No profile update endpoint for theme** — the user settings screen needs to persist the choice to the server.
3. **No server default on login** — after logout/login the client falls back to `localStorage` or system preference, not the server profile.
4. **Web/desktop parity** — both clients read `useSettings`, so a server-backed setting must be loaded into the same context.

### Affected files

- `crates/ruckchat-domain/src/user.rs` — add `theme` field to `User`.
- `migrations/migrations/` — add `users.theme` column.
- `server/openapi.yaml` — add `theme` to `User`, `UpdateProfileRequest`, etc.
- `server/src/repositories/user.rs` — read/write `theme`.
- `server/src/services/user.rs` or `server/src/services/auth.rs` — update profile/theme.
- `desktop/src/hooks/useSettings.ts` — load server theme after login and persist changes back to server.
- `desktop/src/components/Settings.tsx` — same UI, but save to server.
- `web/src/App.tsx` and `web/src/main.tsx` — apply server-loaded theme on boot.

## Open Questions

1. **Field values**
   - `light`, `dark`, `system` (matches current client enum).

2. **API shape**
   - Extend an existing `PUT /me` or profile endpoint.
   - Add `PUT /api/v1/users/me/theme` dedicated endpoint.

3. **Precedence between server theme and localStorage**
   - Server wins after login; localStorage is a fallback when offline.
   - localStorage wins for device-specific theming; server is only a default.

4. **Should theme changes emit real-time updates to other sessions?**
   - No, theme is per-session.
   - Yes, keep all sessions in sync.

## Decisions

- Pending. Needs API design before implementation.

## Status

Open.
