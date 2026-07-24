# ISSUES1 — Light Theme with Light/Dark Toggle

## Source

> Needs a light theme with light/dark toggle — open

## Research Summary

### Current state

- The UI is hardcoded to a dark palette: `bg-gray-900`, `bg-gray-800`, `text-white`, `text-gray-200`, `border-gray-700`, etc. (`desktop/src/components/MessageItem.tsx:73`, `Composer.tsx:163`, `Settings.tsx:25`).
- Tailwind CSS v4 is used (per CLAUDE.md). The project may rely on the default Tailwind dark-mode behavior or no theme switching at all.
- There is no theme provider, CSS-variable layer, or `localStorage` key for theme preference.
- `useSettings.ts` currently persists only `apiUrl` and `notificationsEnabled`; theme is not part of the settings model.

### Gaps

1. **Theme token system** — introduce semantic color tokens (e.g., `--color-bg`, `--color-surface`, `--color-text`) or a Tailwind `dark:` class strategy so components do not hardcode gray values.
2. **Theme state** — add a `theme` value (`light`/`dark`/`system`) to `useSettings.ts` and persist it in `localStorage`.
3. **Toggle UI** — add a light/dark/system toggle in `Settings.tsx` and optionally in the main shell header.
4. **Web/Desktop parity** — both `web/src` and `desktop/src` share `desktop/src/components`, so the theme implementation must work in both builds.
5. **PWA status bar / meta tags** — the web manifest and meta theme-color may need updates for light mode.

### Affected files

- `desktop/src/hooks/useSettings.ts` — add theme preference.
- `desktop/src/components/Settings.tsx` — add theme toggle.
- `desktop/src/components/Shell.tsx` (or `PlatformShell.tsx`) — apply theme class to root element.
- Tailwind configuration / global CSS — define light and dark color tokens.
- `web/public/manifest.json` and `index.html` — update theme-color if needed.

## Open Questions

1. **Which theme strategy should be used?**
   - Tailwind `darkMode: 'class'` with a wrapper class on the root element.
   - CSS custom properties with a single set of semantic classes.
   - A hybrid: Tailwind `dark:` variants plus CSS variables for brand colors.

2. **Should the default be system preference or always dark?**
   - Default to system preference (`prefers-color-scheme`).
   - Default to dark to match the current look.

3. **Which surfaces need the most rework?**
   - Only the chat shell and settings screen first, leaving admin UIs dark for later.
   - Apply the token system to all shared components at once before adding the toggle.

## Decisions

- Theme strategy: hybrid — Tailwind `dark:` variants plus CSS variables for brand/accent colors.
- Default theme: system preference (`prefers-color-scheme`); explicit toggle overrides it.
- Rollout: apply theme tokens to all shared components at once before enabling the toggle.
