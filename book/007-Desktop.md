# 007 - Desktop

## Desktop Client

The desktop client is a Tauri application with a React front end. It targets Linux, macOS, and Windows from a single codebase.

## Technology Stack

| Layer | Technology |
|-------|------------|
| Shell | Tauri (Rust) |
| UI | React + TypeScript |
| Styling | Tailwind CSS |
| State | React hooks + context |
| Routing | React Router or TanStack Router |
| HTTP | Native `fetch` |
| WebSocket | Native `WebSocket` |

## Window Layout

- Primary window is a three-pane chat interface.
- Secondary windows may be opened for thread replies, file previews, or settings.
- Minimum window size: 900 x 600 px.
- Recommended window size: 1280 x 800 px.

## Native Integrations

Tauri exposes these native capabilities:

- **Notifications:** OS-native toast notifications for mentions and DMs.
- **File open/save:** Native dialogs for downloading attachments.
- **Deep links:** `ruckchat://` scheme for organization invitations.
- **Auto-updater:** Check for updates on startup (optional; disabled by default).
- **Tray icon:** Show unread badge and quick status.

## Application State

- `OrganizationStore` — current organization, list of organizations.
- `ChannelStore` — channels and DMs for the active organization.
- `MessageStore` — message history and drafts per conversation.
- `PresenceStore` — online status and typing indicators.
- `NotificationStore` — unread counts and in-app notifications.

## Communication with Server

- REST API for state-changing operations and initial data loads.
- WebSocket for real-time events and presence.
- The WebSocket connection is kept open while the app is running; it reconnects automatically.

## Offline Behavior

- Draft messages are preserved in `localStorage` until sent.
- Failed sends show a retry affordance and remain editable.
- Read positions are cached locally and reconciled on reconnect.

## Build and Release

- Development: `pnpm dev` starts the Vite dev server with Tauri in dev mode.
- Production: `pnpm tauri build` produces platform-specific installers.
- Releases are packaged as `.dmg`, `.AppImage`, `.deb`, `.msi`, and `.exe` where appropriate.

## Security

- The WebView Content Security Policy restricts external requests to the configured server.
- No unsafe-inline scripts.
- Local storage is scoped to the application origin.

## Accessibility

- Full keyboard navigation.
- Screen-reader-friendly labels on message actions and navigation.
- Focus management for modals and thread views.
