# 019 - Web UI

## Browser Client

The Web UI is a Progressive Web App built with Vite, React 19, and Tailwind CSS
v4. It shares components, hooks, contexts, and the API client with the desktop
client, and it is served by the Rust server as static assets.

## Technology Stack

| Layer | Technology |
|-------|------------|
| UI | React 19 + TypeScript 5 |
| Bundler | Vite 6 |
| Styling | Tailwind CSS v4 |
| Routing | React Router v7 |
| State | React hooks + context |
| HTTP | Native `fetch` |
| WebSocket | Native `WebSocket` |
| PWA | Service worker, manifest, offline asset caching |
| Push | Web Push API + server-managed VAPID keys |

## Platform Abstraction

The shared UI code lives in `desktop/src`. Platform-specific behavior is isolated
in `desktop/src/platform/`:

- `desktop` — Tauri shell: tray icon, OS notifications, native file dialog, deep
  links.
- `web` — Browser: Web Push notifications, `<input type="file">` upload, no tray
  or deep links.

`PlatformShell` receives a `Platform` implementation and wires it into the shared
provider tree. This keeps browser-only and desktop-only code out of the shared
components.

## Styling

The Web UI and desktop client share the same Tailwind CSS v4 utility classes.
Tailwind's Vite plugin discovers source files automatically within the project
root, so `web/src/index.css` explicitly registers the shared components with an
`@source` directive:

```css
@import "tailwindcss";
@source "../../desktop/src";
```

`web/src/main.tsx` imports this local stylesheet instead of the one in
`desktop/src/`. Without the `@source` directive, the production `web/dist`
stylesheet omits classes used only in the shared `desktop/src` components,
causing broken layout in the browser.

## Static Asset Serving

The server embeds the contents of `web/dist/` at compile time using
`include_dir!`. Requests under `/{*path}` are served from the embedded directory
unless `web.path` in `ruckchat.yaml` points to a runtime directory. Unknown paths
fall back to `index.html` so React Router can handle client-side routes.

For a production release, build `web/` before `server/`:

```bash
cd web
pnpm install
pnpm build
cd ..
cargo build --release -p ruckchat-server
```

## PWA and Service Worker

- `web/public/manifest.json` describes the app for install prompts.
- `web/public/sw.js` caches static assets and handles `push` and
  `notificationclick` events.
- `web/src/service-worker/register.ts` registers `/sw.js` on startup.

The service worker only caches same-origin `GET` requests and static assets.
Outgoing messages are not queued offline in this phase; the desktop draft
persistence behavior is not replicated in the browser.

## Web Push

The server stores a VAPID key pair in `ruckchat.yaml` under `web_push`:

```yaml
web_push:
  vapid_private_key: "..."
  vapid_public_key: "..."
```

The browser:

1. Requests the public key from `GET /web-push/vapid-key`.
2. Subscribes through the Push API using that key.
3. Sends the subscription to `POST /web-push/subscribe`.

When a `message.created` event occurs, the server filters subscriptions for:

- Direct-message recipients.
- Users whose IDs appear in the message's `mentioned_user_ids` array.

Mentions are stored as first-class Tiptap `mention` nodes with `id` (user ID)
and `label` (display name) attributes. Channel members who are not mentioned do
not receive push notifications.

## Cross-Origin Support

The server CORS layer is explicit and credentials-aware. Allowed origins default
to the origin of `base_url`; additional origins can be listed in
`web.allowed_origins`. The desktop client does not need CORS because the web
view loads the UI from the same dev server it proxies; the API and WebSocket
both use `credentials: 'include'` so the `ruckchat_session` cookie works for
the web app.

## File Uploads

The browser composer uses a multipart `POST /files` upload that writes file bytes
to `files.directory`. The desktop composer records metadata via `POST /files/record`
for files selected through the Tauri file dialog. Both endpoints attach the
resulting file records to messages with `POST /messages/{id}/attachments`.

## Responsive Layout

The shared `Shell` and `Sidebar` support a mobile breakpoint: a hamburger button
opens the sidebar as a fixed overlay on small screens, and navigation links close
the overlay after selection. The message pane and composer remain usable down to
narrow viewports.

## Development

```bash
cd web
pnpm install
pnpm dev
```

The Vite dev server proxies API requests and WebSocket upgrades to
`http://localhost:3000`.

## Testing

- `pnpm typecheck` and `pnpm test` run in both `desktop/` and `web/`.
- `desktop/` unit tests cover the shared components and hooks.
- `web/` currently has no component tests; the shared tests in `desktop/` exercise
  the same code paths.

## Related

- `docs/ADR-011-Web-UI.md`
- `desktop/README.md`
- `web/README.md`
- `server/README.md`
