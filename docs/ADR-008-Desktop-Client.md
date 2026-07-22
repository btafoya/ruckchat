# ADR 008: Desktop Client

## Status

Accepted

## Context

RuckChat needs a native desktop client for Linux, macOS, and Windows. The
client must share behavior with the future mobile client wherever possible, reuse
the existing Rust server (REST API and WebSocket), and provide native OS
integrations such as notifications, file dialogs, deep links, and a tray icon.

Key constraints:

- The desktop client must be built from a single codebase that targets all three
  major desktop platforms.
- It must communicate with the server using the same authenticated REST and
  WebSocket transports as other clients.
- Native capabilities should be exposed through a stable, well-supported bridge.
- The project should minimize additional runtime dependencies and avoid separate
  services beyond the existing Axum server and PostgreSQL database.

## Decision

Build the desktop client with **Tauri v2** as the native shell and **React 19**
with **TypeScript** for the user interface. The project lives in `desktop/` and
its Rust shell crate (`desktop/src-tauri`) is included in the top-level Cargo
workspace.

Specific choices:

1. **Shell**: Tauri v2 provides the WebView window, native API bridge, and
   cross-platform bundling. It uses the platform WebView on each OS, avoiding
   the need to ship a separate browser engine.

2. **Frontend**: React 19, TypeScript, Vite, Tailwind CSS v4, and React Router
   v7. State management uses React hooks and context, matching the convention
   documented in `book/007-Desktop.md`.

3. **Workspace integration**: `desktop/src-tauri` is a member of the top-level
   Cargo workspace. It inherits workspace metadata and lints, so `cargo check
   --workspace` and `cargo clippy --workspace` include the desktop crate.

4. **Communication**: The frontend uses native `fetch` for REST and native
   `WebSocket` for real-time events, authenticated with the same session cookie
   or bearer token as the server.

5. **Native integrations**: Tauri plugins expose notifications, file open
   dialogs, deep-link handling for `ruckchat://` invitations, and a tray icon.
   A `/settings` screen lets users change the backend URL and toggle
   notifications.

6. **Offline resilience**: Draft messages and read positions are cached locally
   in `localStorage` and reconciled on reconnect. Failed sends remain in the
   message list with a retry affordance.

7. **Security**: The Tauri Content Security Policy restricts `connect-src` to the
  configured RuckChat server, disallows `unsafe-inline` scripts, and scopes
  local storage to the application origin.

## Consequences

- Web frontend skills and components can be reused between desktop and the future
  web or mobile clients.
- Tauri bundles platform-specific installers (`.dmg`, `.AppImage`, `.deb`,
  `.msi`, `.exe`) from a single codebase.
- The WebView shell depends on OS-provided WebView runtimes, which reduces
  bundle size but creates a small compatibility testing matrix.
- Adding `desktop/src-tauri` to the Cargo workspace increases initial compile
  time for `cargo check --workspace` and requires all workspace lints to pass
  in the desktop crate.
- Icons and installer metadata must be generated before release builds; a
  placeholder icon set is generated during the scaffold task.
