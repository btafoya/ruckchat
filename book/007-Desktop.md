# 007 - Desktop

## Desktop Client

The desktop client is a Tauri application with a React front end. It targets Linux, macOS, and Windows from a single codebase.

## Technology Stack

| Layer | Technology |
|-------|------------|
| Shell | Tauri v2 (Rust) |
| UI | React 19 + TypeScript 5 |
| Bundler | Vite 6 |
| Styling | Tailwind CSS v4 |
| State | React hooks + context |
| Routing | React Router v7 |
| HTTP | Native `fetch` |
| WebSocket | Native `WebSocket` |
| Tests | Vitest + React Testing Library |

## Window Layout

- Primary window is a three-pane chat interface.
- Secondary windows may be opened for thread replies, file previews, or settings.
- Minimum window size: 900 x 600 px.
- Recommended window size: 1280 x 800 px.

## Native Integrations

Tauri exposes these native capabilities:

- **Notifications:** OS-native toast notifications for mentions and DMs, gated by
  a user toggle in the settings screen.
- **File open:** Native multi-select dialog via `@tauri-apps/plugin-dialog` for
  choosing file metadata attachments in the composer.
- **Deep links:** `ruckchat://` scheme for organization invitations. The Tauri
  shell registers the scheme, emits open events while running, and exposes a
  `get_deep_link_url` command so the React front end can read the startup URL.
- **Auto-updater:** Check for updates on startup (optional; disabled by default).
- **Tray icon:** System tray icon with a Show/Quit context menu and a tooltip that
  reflects the unread count. The unread badge is updated from a `set_unread_count`
  Tauri command called whenever the total unread count changes.

## Application State

State is managed through React context providers backed by custom hooks:

- `SessionContext` — authenticated user, access token, and session actions.
- `OrganizationContext` — current organization and list of organizations.
- `ChannelContext` — channels for the active organization.
- `DirectMessageContext` — direct-message conversations for the active organization.
- `MessageContext` — message history, reactions, and thread replies per conversation.
- `PresenceContext` — online status for organization members.
- `TypingContext` — active typing indicators per conversation.
- `RealtimeContext` — WebSocket connection status and server event dispatch.
- **Settings** — `useSettings` hook (not a context) persisting the backend URL and
  notification preference to `localStorage`. API hooks and the WebSocket hook
  read this URL and pass it to the fetch/WebSocket clients.

Each store exposes refresh actions that call the REST API and update local state.

## Communication with Server

- REST API for state-changing operations and initial data loads.
- Native `WebSocket` for real-time events and presence.
- The WebSocket connection is kept open while the app is running and reconnects
  automatically with exponential backoff (500 ms to 30 s).
- Server events are dispatched into the appropriate React stores:
  - `message.created`, `message.updated`, `message.deleted` → `MessageContext`
  - `reaction.added`, `reaction.removed` → `MessageContext` (reactions are cached
    locally because the `Message` schema does not include them)
  - `presence.updated` → `PresenceContext`
  - `typing.updated` → `TypingContext` (the server emits a single typing pulse;
    the client clears typing users after a short timeout)

## Offline Behavior

- Draft messages are preserved per conversation in `localStorage`
  (`ruckchat_draft_${conversationId}`) and restored when the conversation is
  selected.
- Sends optimistically add a pending message with a `pending-` prefixed ID. If the
  request fails, the pending message stays in place with a retry affordance in
  `MessageItem`. `useMessages` exposes `retryMessage` to resend the original
  content once the connection or server is healthy again.
- Read positions are cached locally and reconciled on reconnect.

## Project Layout

```text
desktop/
├── package.json              # pnpm scripts and dependencies
├── vite.config.ts           # Vite + Tailwind + Vitest configuration
├── tsconfig.json            # TypeScript project settings
├── index.html               # Vite entry point
├── src/                     # React + TypeScript frontend
│   ├── api/                 # OpenAPI-generated types, fetch client, and API modules
│   │   ├── schema.ts
│   │   ├── client.ts
│   │   ├── events.ts
│   │   └── ...
│   ├── components/          # UI components (Shell, Sidebar, MessagePane,
│   │   │                     Composer, MessageItem, ThreadPane, Settings, etc.)
│   ├── context/             # React context providers for state stores
│   │   ├── SessionContext.tsx
│   │   ├── OrganizationContext.tsx
│   │   ├── ChannelContext.tsx
│   │   ├── DirectMessageContext.tsx
│   │   ├── MessageContext.tsx
│   │   ├── PresenceContext.tsx
│   │   ├── TypingContext.tsx
│   │   └── RealtimeContext.tsx
│   ├── hooks/               # State hooks and real-time sync
│   │   ├── useSession.ts
│   │   ├── useOrganizations.ts
│   │   ├── useChannels.ts
│   │   ├── useDirectMessages.ts
│   │   ├── useMessages.ts
│   │   ├── usePresence.ts
│   │   ├── useTyping.ts
│   │   ├── useUnread.ts
│   │   ├── useWebSocket.ts
│   │   ├── useRealtimeStore.ts
│   │   ├── useSettings.ts
│   │   ├── useNotifications.ts
│   │   ├── useTray.ts
│   │   └── useDeepLink.ts
│   ├── main.tsx
│   ├── App.tsx
│   ├── App.test.tsx
│   ├── index.css
│   └── test/setup.ts
└── src-tauri/               # Tauri Rust shell
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    ├── src/lib.rs
    ├── src/main.rs
    ├── capabilities/default.json
    └── icons/
```

## Settings

The `/settings` route displays a settings screen where users can:

- Change the backend URL used by REST and WebSocket connections. An empty value
  falls back to `http://localhost:3000`.
- Enable or disable OS notifications for mentions and DMs.

The settings object is stored under `ruckchat_settings` in `localStorage` and
read by `useSession`, the API hooks, and `useWebSocket` at runtime.

## Build and Release

- Development: `pnpm tauri dev` starts the Vite dev server and opens a Tauri
  WebView in dev mode.
- Type check: `pnpm typecheck`
- Unit tests: `pnpm test`
- Production: `pnpm tauri build` produces platform-specific installers.
- Releases are packaged as `.dmg`, `.AppImage`, `.deb`, `.msi`, and `.exe` where appropriate.

The desktop crate is included in the top-level Cargo workspace, so
`cargo check --workspace` and `cargo clippy --workspace` also cover
`desktop/src-tauri`.

## Security

- The WebView Content Security Policy restricts external requests to the configured server.
- No unsafe-inline scripts.
- Local storage is scoped to the application origin.

## Accessibility

- Full keyboard navigation.
- Screen-reader-friendly labels on message actions and navigation.
- Focus management for modals and thread views.
