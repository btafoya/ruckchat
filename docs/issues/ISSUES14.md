# ISSUES14 — Admin Menu Items in Top Bar with Icons

## Source

> Admin menu items should move to the top bar far left using icons (from https://fontawesome.com/search?ic=free-collection) with mouseover tool tips to show the full menu item title — open

## Research Summary

### Current state

- `ServerAdminShell.tsx` renders a left sidebar with text tabs: Organizations, Users, Admins, Settings, Audit Log (`desktop/src/components/admin/ServerAdminShell.tsx:5-11`).
- `OrgAdminShell.tsx` renders a similar left sidebar with text tabs: Settings, Members, Roles, Permissions, Emoji, Teams (`desktop/src/components/admin/OrgAdminShell.tsx:6-13`).
- Both shells have a top header bar that currently only shows the page title and a Back link.
- Neither shell uses Font Awesome or icon-based navigation.

### Gaps

1. **No icon representation of admin tabs** — each tab needs a Font Awesome free icon and a tooltip that reveals the full label on hover.
2. **Top bar placement** — the icons must move to the far left of the existing top header bar, before the page title.
3. **Active state styling** — icon buttons need an active indicator so users know which tab is selected.
4. **Mobile behavior** — the existing collapsible mobile nav in `OrgAdminShell` should be reconsidered if icons are in the top bar; either replace the mobile drawer with the icon bar or keep both.
5. **Accessibility** — icon-only navigation requires `aria-label` and tooltips; keyboard focus must show the tooltip.

### Affected files

- `desktop/src/components/admin/ServerAdminShell.tsx` — replace or supplement sidebar tabs with top-bar icon buttons.
- `desktop/src/components/admin/OrgAdminShell.tsx` — same for org admin.
- `desktop/package.json` / `web/package.json` — add Font Awesome dependency or use inline SVGs.
- `desktop/src/components/admin/index.ts` — no changes unless new icon components are added.

## Open Questions

1. **Icon set choice**
   - Use `@fortawesome/free-solid-svg-icons` via React bindings.
   - Use inline SVGs copied from Font Awesome to avoid an extra dependency.

2. **Layout on mobile**
   - Icons collapse into a scrollable horizontal bar.
   - Keep the hamburger drawer with text labels on mobile.

3. **Does the left sidebar stay or get removed entirely?**
   - Remove the sidebar and rely solely on the top icon bar.
   - Keep a slim icon sidebar instead of a top bar.

4. **Which icon maps to which tab?**
   - Organizations: `building`.
   - Users/Admins: `user-shield` or `users`.
   - Settings: `gear`.
   - Audit Log: `clipboard-list`.
   - Members: `users`.
   - Roles/Permissions: `user-lock` / `key`.
   - Emoji: `face-smile`.
   - Teams: `people-group`.

## Decisions

- Pending. Need to decide whether to replace the sidebar entirely or add a complementary top icon bar.

## Status

Open.
