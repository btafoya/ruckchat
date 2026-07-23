# RuckChat Desktop

Tauri v2 + React + TypeScript desktop client for RuckChat.

## Development

```bash
cd desktop
pnpm install
pnpm tauri dev
```

The frontend dev server runs on `http://localhost:5173` and Tauri opens a
WebView pointing to it. The Rust side rebuilds automatically when
`src-tauri/src` changes.

## Build

```bash
pnpm tauri build
```

Installers are produced under `src-tauri/target/release/bundle/`.

## Release builds

Pushing a Git tag matching `v*` triggers `.github/workflows/release.yml`, which
builds cross-platform installers:

- Linux: `.deb` and AppImage on `ubuntu-22.04`
- macOS: `.dmg` on `macos-latest`
- Windows: `.msi` and NSIS on `windows-latest`

The workflow uses `tauri-apps/tauri-action` and attaches the bundles to a GitHub
release named after the tag. Unsigned installers are produced unless repository
secrets for code signing are configured.

Generate application icons before producing release installers:

```bash
pnpm tauri icon /path/to/source.png
```

## Project Layout

- `src/api/` — OpenAPI-generated schema, fetch client, and API modules.
- `src/components/` — UI components (`Shell`, `Sidebar`, `MessagePane`,
  `Composer`, `MessageItem`, `ThreadPane`, `Settings`, `AuthScreen`, `AuthForm`).
- `src/context/` — React context providers for session, organizations, channels,
  direct messages, messages, presence, typing, and real-time sync.
- `src/hooks/` — State hooks and the WebSocket connection manager, plus
  `useSettings`, `useNotifications`, `useTray`, and `useDeepLink`.
- `src/platform/` — Platform abstraction layer. The `desktop` and `web` shims
  provide platform-specific implementations of notifications, file pickers, tray,
  and deep links so the same components work in Tauri and the browser.
- `src-tauri/` — Tauri Rust shell and native integrations.
- `index.html` — Vite entry point.
- `vite.config.ts` — Vite + Tailwind + Vitest configuration.

## Technology Choices

- **Shell:** Tauri v2
- **UI:** React 19, TypeScript, Tailwind CSS v4
- **Routing:** React Router v7
- **State:** React hooks + context
- **HTTP:** Native `fetch`
- **WebSocket:** Native `WebSocket`
- **Tests:** Vitest + React Testing Library

## Notes

- The backend URL defaults to `http://localhost:3000` and can be changed from
  the `/settings` screen. The URL is stored under `ruckchat_settings` in
  `localStorage` and used by all API hooks and the WebSocket connection.
- Messaging features implemented in `MessagePane`, `Composer`, `MessageItem`,
  and `ThreadPane` include paginated history loading, message sending with
  optimistic updates, failed-send retry, `@mention` autocomplete derived from
  direct-message member IDs, markdown preview, typing indicators, reactions,
  file metadata attachments, thread replies, and local unread badges.
- Native integrations use Tauri plugins: notifications (`useNotifications`),
  file dialogs in the composer, deep-link registration for `ruckchat://`, and a
  tray icon with an unread count tooltip (`set_unread_count` command). The browser
  version of these features lives in `src/platform/web.tsx` and uses the Web Push
  API and a standard `<input type="file">` instead of Tauri plugins.
- Draft messages are persisted per conversation (`ruckchat_draft_${id}` in
  `localStorage`).
- TypeScript API types are generated from `../server/openapi.yaml` into
  `src/api/schema.ts`. Regenerate with `pnpm dlx openapi-typescript
  ../server/openapi.yaml -o src/api/schema.ts` when the server contract changes.
- Application icons must be generated with `pnpm tauri icon <source.png>` before
  producing release installers.
