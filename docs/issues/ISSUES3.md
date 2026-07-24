# ISSUES3 — Single-Organization Auto-Redirect to #general

## Source

> If the user belongs to one organization it should take them directly to the #general channel when logging in — open

## Research Summary

### Current state

- After login the router lands on `/*` and the index route navigates to `/org` (`desktop/src/PlatformShell.tsx:192-194`).
- `/org` renders an empty `div`, so a user with one organization sees a blank chat surface.
- `useOrganizations` loads the user's organizations; `AuthenticatedShell` already has access to that list before rendering `Shell` (`PlatformShell.tsx:57`, `94-110`).
- The router defines routes for `/org/:organizationId/channel/:channelId` and `/org/:organizationId/dm/:dmId` (`PlatformShell.tsx:195-201`).
- Channel data are loaded by `useChannels` once an `organizationId` is known (`PlatformShell.tsx:69`).

### Gaps

1. **Redirect logic** — in `AuthenticatedShell` (or the router index route), detect when the user belongs to exactly one organization and redirect to its `#general` channel.
2. **General channel lookup** — the redirect needs the `general` channel ID for the organization; this requires `useChannels` to finish loading or a deterministic slug/ID.
3. **Multi-organization fallback** — if the user belongs to zero or multiple organizations, keep the current org picker behavior.
4. **Deep link / refresh safety** — ensure the redirect does not fight with a deep-linked URL such as `/org/:id/channel/:channelId`.

### Affected files

- `desktop/src/PlatformShell.tsx` — add redirect logic inside `AuthenticatedShell` or replace the `/org` placeholder.
- `desktop/src/hooks/useOrganizations.ts` — may need loading state exposure if not already present.
- `desktop/src/hooks/useChannels.ts` — may need a way to find the default channel by name.

## Open Questions

1. **Which trigger should perform the redirect?**
   - A `useEffect` inside `AuthenticatedShell` that redirects after organizations load.
   - A dedicated route component rendered at `/org` that decides the destination.

2. **How is the default channel identified?**
   - By name `general` (case-insensitive), which is created at organization creation.
   - By a flag such as `is_default` on the `Channel` schema.

3. **Should the redirect happen only on initial login or on every navigation to `/org`?**
   - Every time the user lands on `/org`.
   - Only once after login (then allow explicit navigation to `/org`).

## Decision

- Redirect every time the authenticated user lands on `/org` if they belong to exactly one organization.
- Default channel: use `general` on first visit; otherwise return to the last selected channel persisted in `localStorage` or router history.
