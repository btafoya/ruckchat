# ISSUES13 — Sidebar Missing on Mobile

## Source

> The sidebar is missing on mobile — open

## Research Summary

### Current state

- `Sidebar.tsx` uses `hidden md:flex` on desktop and a fixed slide-over when `mobileOpen` is true (`desktop/src/components/Sidebar.tsx:107-109`).
- `Shell.tsx` must provide `mobileOpen` and `onClose` to the sidebar; without those props the sidebar is hidden on narrow viewports.
- `OrgAdminShell.tsx` has a mobile hamburger toggle and a working mobile drawer (`desktop/src/components/admin/OrgAdminShell.tsx:68-93`), but the main `Shell` for chat does not appear to expose a comparable toggle.
- The main chat route (`/org/:organizationId/channel/:channelId`) renders a placeholder `<div />` in `PlatformShell.tsx`, meaning the actual `MessagePane` is rendered inside `Shell`.

### Gaps

1. **No visible mobile entry point** — on a narrow screen there is no hamburger menu to open the sidebar, so the user cannot switch channels/DMs or see unread counts.
2. **No overlay close behavior for chat shell** — `Sidebar` supports `mobileOpen`/`onClose`, but the host must provide them.
3. **Header inconsistency** — the chat shell lacks the mobile header pattern used by the admin shells.

### Affected files

- `desktop/src/components/Shell.tsx` — add mobile header with hamburger toggle and pass `mobileOpen`/`onClose` to `Sidebar`.
- `desktop/src/components/Sidebar.tsx` — already accepts mobile props; may need styling tweaks for full-screen mobile drawer.
- `desktop/src/components/MessagePane.tsx` — ensure the message pane header does not conflict with the shell header.

## Open Questions

1. **Should the mobile sidebar be a full-screen drawer or a partial sheet?**
   - Full-screen drawer (matches `OrgAdminShell`).
   - Partial sheet leaving the current channel visible.

2. **Where does the hamburger button live?**
   - In a new top bar inside `Shell.tsx`.
   - Inside `MessagePane.tsx` header (closer to the active conversation).

3. **Should swipe gestures open/close the sidebar?**
   - Yes, common mobile pattern.
   - No, keep it explicit to avoid accidental navigation.

## Decisions

- The hamburger toggle, `mobileOpen`/`onClose` wiring, and full-screen mobile drawer were already implemented in `Shell.tsx`/`Sidebar.tsx` during earlier work, so no additional changes were required.

## Status

Completed (already implemented).
