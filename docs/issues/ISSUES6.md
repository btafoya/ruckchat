# ISSUES6 — Back-to-Chat Link in Admin UIs

## Source

> Server Admin UI and Organization Admin UI both have no link to return to the chat UI — open

## Research Summary

### Current state

- `Settings.tsx` already has a "Back" `NavLink` to `/` (`desktop/src/components/Settings.tsx:28-30`).
- `ServerAdminShell` and `OrgAdminShell` are the layout wrappers for admin routes (`desktop/src/PlatformShell.tsx:175-191`).
- Neither admin shell currently exposes an obvious back-to-chat link in the available snippets.
- The admin routes are isolated under `/admin/server/*` and `/org/:organizationId/admin/*`.

### Gaps

1. **Server admin shell** — add a persistent link back to the main chat UI (`/`).
2. **Org admin shell** — add a persistent link back to the active organization's chat (`/org/:organizationId/channel/:channelId` or `/`).
3. **Keyboard / a11y** — ensure the link is reachable and labeled clearly (e.g., "Back to RuckChat").
4. **Mobile / PWA** — ensure the back link is visible on narrow viewports.

### Affected files

- `desktop/src/components/admin/ServerAdminShell.tsx` — add back link in header.
- `desktop/src/components/admin/OrgAdminShell.tsx` — add back link in header.
- `desktop/src/components/admin/index.ts` — export if new shared header component is created.

## Open Questions

1. **Where should the back link live?**
   - In the top-left of the admin shell header.
   - As a breadcrumb next to the current admin section title.

2. **What should the destination be?**
   - The root chat route `/`, which currently redirects to the org picker or single org.
   - The most recently active organization/channel if known from `localStorage` or router history.

3. **Should the link be an icon, text, or both?**
   - Text only ("← Back to chat").
   - Icon + text for clarity at all sizes.

## Decisions

- Back-to-chat destination: most recently active channel, persisted in `localStorage` or router history.
- Back link placement: top-right "Back" link, matching `Settings.tsx` style.
