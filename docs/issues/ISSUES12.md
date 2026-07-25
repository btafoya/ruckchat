# ISSUES12 — Collapsible Sidebar (Full ↔ Narrow)

## Source

> The sidebar needs to be able to collapse to from full to narrow using a collapse icon — open

## Research Summary

### Current state

- `Sidebar.tsx` renders a fixed-width `w-64` aside on desktop and a fixed `w-64` slide-over on mobile (`desktop/src/components/Sidebar.tsx:107-109`).
- There is no narrow/collapsed state; the sidebar is either fully open or hidden.
- The only mobile close affordance is the `✕` button inside the header, which closes the mobile overlay entirely (`desktop/src/components/Sidebar.tsx:116-124`).
- `Shell.tsx` hosts the sidebar and the main message pane; it does not currently manage a collapsed sidebar state.

### Gaps

1. **No collapsed width state** — need a persistent narrow mode (e.g., `w-16`) showing channel/DM icons or initials only.
2. **No collapse toggle button** — a collapse/expand icon is required, likely at the top of the sidebar.
3. **No persistence** — the collapsed preference should be stored (localStorage or server profile).
4. **Mobile behavior undefined** — on small screens the sidebar currently overlays the full screen; a narrow dock may not make sense and should probably stay as the existing mobile drawer.
5. **Content adaptation** — channel names, DM labels, and section headers must be truncated or replaced with icons when collapsed.

### Affected files

- `desktop/src/components/Sidebar.tsx` — add collapsed state, toggle button, and narrow rendering.
- `desktop/src/components/Shell.tsx` — pass collapsed state and adjust main content layout.
- `desktop/src/hooks/useSettings.ts` — persist the collapse preference if stored locally.
- `desktop/src/lastConversation.ts` — unaffected, but navigation after selection should still work in narrow mode.

## Open Questions

1. **What is the collapsed width?**
   - `w-16` (64 px) showing icons/avatars only.
   - `w-20` (80 px) with room for labels below icons.

2. **What does the narrow sidebar show for channels and DMs?**
   - First letter or a hash (`#`) for channels.
   - Member avatar initials for DMs.
   - Keep section headers or collapse them into tooltips.

3. **Where does the toggle live?**
   - Top of the sidebar header, next to the RuckChat title.
   - Bottom of the sidebar near settings/logout.

4. **Should the state persist per-device or per-user?**
   - Per-device in `localStorage` (simpler).
   - Per-user on the server (aligns with ISSUES17 theme-as-profile).

## Decisions

- Collapsed width: `w-16` (64 px) showing two-letter initials for orgs/channels, initials for DMs, and two-letter abbreviations for admin/settings links.
- Toggle placement: in the sidebar header, replacing the RuckChat title when collapsed and sitting next to the sign-out control when expanded.
- Persistence: per-device in `localStorage` via `useSettings.ts` (`sidebarCollapsed` boolean), since ISSUES17 has not landed yet.
- Mobile behavior: unchanged; the existing full-screen slide-over from ISSUES13 remains the mobile experience.

## Status

Completed.
