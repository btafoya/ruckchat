# RuckChat Web UI

Browser-based Progressive Web App that shares the React components, hooks, and
API client from the desktop client via the `desktop/src/platform/` abstraction.

## Development

```bash
pnpm install
pnpm dev
```

The dev server proxies API requests and WebSocket upgrades to
`http://localhost:3000`.

## Production build

```bash
pnpm build
```

Output is written to `dist/`. The Rust server embeds `dist/` at compile time, or
serves it from a configured `web.path` directory. For a production release, build
`web/` before `server/` so the Rust binary includes the latest assets:

```bash
cd web
pnpm build
cd ..
cargo build --release -p ruckchat-server
```

## PWA and Web Push

- `public/manifest.json` describes the app for browser install prompts.
- `public/sw.js` caches static assets, displays push notifications, and handles
  notification clicks.
- `src/service-worker/register.ts` registers `/sw.js` on startup.
- `desktop/src/platform/web.tsx` requests notification permission, fetches the
  VAPID public key from `GET /web-push/vapid-key`, and subscribes through the
  browser Push API. The subscription is sent to `POST /web-push/subscribe`.

Push notifications are delivered for direct messages and `@mentions` only.

## Responsive Layout

The shared `Shell` and `Sidebar` render a hamburger menu on narrow viewports that
opens the sidebar as a fixed overlay. The message pane and composer adapt to the
available width.

## Type checking and tests

```bash
pnpm typecheck
pnpm test
```

## Related

- `desktop/README.md`
- `server/README.md`
- `docs/ADR-011-Web-UI.md`
- `book/019-Web-UI.md`
