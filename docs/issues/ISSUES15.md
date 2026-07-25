# ISSUES15 — Single-Organization Home Redirect

## Source

> On login if the user only belongs to one organization go straight to that organization when they are going to the 'home' view - basically the org view becomes the home view — open

## Research Summary

### Current state

- `PlatformShell.tsx` routes `/*` to `AuthenticatedShell`, whose index route redirects to `/org` (`desktop/src/PlatformShell.tsx:277-279`).
- `/org` renders `OrgIndexRoute`, which currently appears as an empty placeholder in the source shown earlier; the original Phase 3 implementation added logic to redirect single-org users to the last-selected channel or `#general` (`docs/issues/WORKFLOW.md` Phase 3).
- The issue is reported as still open, suggesting the redirect may have regressed or never fully covered the auth success landing.
- `AuthScreen.tsx` redirects to `/` after login (`desktop/src/components/AuthScreen.tsx:37`), which then hits the index redirect.

### Gaps

1. **Auth success path may not redirect deeply enough** — after login the user lands on `/` and then the index route sends them to `/org`, but if `OrgIndexRoute` is empty the single-org user sees a blank screen.
2. **No server-side or client-side default organization stored on login** — the session response could include a `default_organization_id` to avoid a second redirect hop.
3. **Multi-organization fallback** — when the user has zero or multiple organizations, `/org` should still act as the organization picker.
4. **Post-login navigation edge cases** — direct navigation to `/` after login, deep links, and refresh must all behave correctly.

### Affected files

- `desktop/src/components/AuthScreen.tsx` — redirect destination after login.
- `desktop/src/PlatformShell.tsx` — `OrgIndexRoute` and `AuthenticatedShell` index route.
- `desktop/src/lastConversation.ts` — last-selected channel lookup.
- `desktop/src/hooks/useOrganizations.ts` — loading state and default org.

## Open Questions

1. **Should the redirect happen in `AuthScreen` or in the router index route?**
   - `AuthScreen`: simpler, but it needs organization data before redirecting.
   - Router index route / `OrgIndexRoute`: keeps routing logic centralized.

2. **What is the destination for a single-org user?**
   - Last selected channel/DM if available.
   - The organization's `#general` channel unconditionally.

3. **Should the backend return a `default_organization_id` in the session?**
   - Yes, avoids an extra client round-trip.
   - No, keep the client responsible.

## Decisions

- Redirect logic lives in `OrgIndexRoute` (`desktop/src/PlatformShell.tsx`) rather than in `AuthScreen`, keeping routing decisions centralized.
- A single-organization user navigating to `/org` is redirected immediately to `/org/{organizationId}`.
- Multi-organization and zero-organization users see the `OrgIndex` picker/empty-state view at `/org`.
- `AuthScreen` continues to redirect to `/` on success; the existing index route chain (`/ -> /org -> /org/{id}`) handles the rest.

## Status

Implemented.
