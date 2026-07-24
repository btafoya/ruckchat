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

Each build injects a unique `CACHE_NAME` (`injectServiceWorkerHash` in
`web/vite.config.ts`). On `install`, the worker checks whether any other
cache name already exists: if so, this is a genuine update (not a first-ever
install), and on `activate` it force-navigates every open window client to
its current URL after `clients.claim()`. This is necessary because an
already-open tab's in-memory JavaScript has no way to notice a new deploy on
its own — without it, a tab left open across a redeploy keeps running the
old bundle indefinitely. Service workers require a secure context, so this
only applies when the Web UI is served over HTTPS or from `localhost`; a
plain-HTTP LAN address (e.g. `http://192.168.x.x:3922`) has no service
worker at all, and a stale tab there needs a manual reload.

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

## Spell Checking

The Tiptap composer (`desktop/src/components/Composer.tsx`, shared by the web
client) uses `@farscrl/tiptap-extension-spellchecker` for inline spell-check
underlines and suggestion popups. The extension does not implement checking
itself; `desktop/src/spelling/SpellingProofreader.ts` implements its
`IProofreaderInterface` and calls the server:

- `POST /api/v1/spelling/check` — checks a block of text and returns
  misspellings with byte offsets and suggestions.
- `POST /api/v1/spelling/suggest` — returns suggestions for a single word.
  Results are cached client-side for one minute, keyed by the
  diacritic-stripped, lowercased word.
- `GET /api/v1/spelling/languages` — lists supported language tags.

The server-side engine lives in `crates/ruckchat-spelling` and wraps the
pure-Rust [`spellbook`](https://crates.io/crates/spellbook) Hunspell
implementation with embedded LibreOffice `en-US` `.aff`/`.dic` dictionaries
(`include_str!`), avoiding a C++ toolchain dependency at build time.
`server/src/services/spelling.rs` adds per-user token-bucket rate limiting
(10 requests/second burst, 100/minute) on top of the engine. The feature is
gated by the `spelling_enabled` and `spelling_default_language` server
settings (`server/src/services/server_settings.rs`); when disabled, the
endpoints return empty results instead of an error so the composer degrades
silently. See `docs/ADR-014-Spell-Checker.md` for the embedding decision.

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
