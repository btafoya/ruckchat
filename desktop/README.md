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

## Project Layout

- `src/api/` — OpenAPI-generated schema, fetch client, and API modules.
- `src/components/` — UI components (`Shell`, `Sidebar`, `MessagePane`,
  `Composer`, `MessageItem`, `ThreadPane`, `AuthScreen`, `AuthForm`).
- `src/context/` — React context providers for session, organizations, channels,
  direct messages, messages, presence, typing, and real-time sync.
- `src/hooks/` — State hooks and the WebSocket connection manager.
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

- The backend URL is currently hard-coded to `http://localhost:3000` for
  development (matching the server default in `ruckchat-config`). A settings
  screen will make this configurable in the native integrations task.
- Messaging features implemented in `MessagePane`, `Composer`, `MessageItem`,
  and `ThreadPane` include paginated history loading, message sending with
  optimistic updates, `@mention` autocomplete derived from direct-message
  member IDs, markdown preview, typing indicators, reactions, file metadata
  attachments, thread replies, and local unread badges.
- TypeScript API types are generated from `../server/openapi.yaml` into
  `src/api/schema.ts`. Regenerate with `pnpm dlx openapi-typescript
  ../server/openapi.yaml -o src/api/schema.ts` when the server contract changes.
- Application icons must be generated with `pnpm tauri icon <source.png>` before
  producing release installers.
