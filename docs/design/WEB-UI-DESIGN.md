# Web UI Design — Phase 10

## 1. Context and Goals

RuckChat already has a Tauri + React desktop client under `desktop/`. The goal of
Phase 10 is to ship a browser-based Web UI that reuses the same React
components, hooks, and API client, while adding Progressive Web App (PWA)
capabilities and server-side static asset delivery.

### Goals

- Let users run RuckChat in a browser without installing the desktop app.
- Reuse as much of `desktop/src` as possible to avoid duplicate UI code.
- Support single-binary/self-hosted deployments by serving the Web UI from the
  Rust server.
- Support PWA install, offline resilience, and browser push notifications.
- Keep the existing desktop client unchanged except for a small platform
  abstraction layer.

### Non-Goals

- A separate mobile-native redesign (that is the Flutter Phase 11).
- Replacing the desktop client.
- Real-time video/voice (out of scope).

## 2. High-Level Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                         Clients                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Desktop   │  │    Web      │  │  Mobile (Flutter)   │ │
│  │ Tauri/React │  │   React/Vite│  │     Phase 11        │ │
│  │             │  │   PWA/SW    │  │                     │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬────────────┘ │
│         │                │                    │            │
│         └────────────────┴────────────────────┘            │
│                          │                                   │
│           Shared React code lives in `desktop/src`            │
│          (components, hooks, API client, contexts)           │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                     RuckChat Server (Axum)                   │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │   REST API   │  │  WebSocket   │  │ Static Web Assets  │  │
│  │  (existing)  │  │  (existing)  │  │  (new, Phase 10)   │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Web Push endpoints + subscriptions repository (new)       │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 3. Directory Structure

```text
root/
├── desktop/
│   ├── src/
│   │   ├── platform/           # NEW — platform abstraction entry point
│   │   │   ├── index.ts        #     exports platform contract
│   │   │   ├── desktop.ts      #     Tauri implementations
│   │   │   └── web.ts          #     browser implementations (used by web/)
│   │   ├── App.tsx             #     updated to inject desktop platform hooks
│   │   └── ...                 #     shared components, hooks, API, contexts
│   └── src-tauri/              # unchanged
├── web/                        # NEW — Vite web build
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   ├── public/
│   │   ├── manifest.json
│   │   ├── icons/
│   │   └── sw.js               # generated or hand-written service worker
│   └── src/
│       ├── main.tsx
│       ├── App.tsx              # injects web platform hooks
│       ├── service-worker/
│       │   └── register.ts
│       └── README.md
└── server/
    └── src/
        ├── handlers/
        │   └── web_assets.rs    # NEW — fallback static asset serving
        ├── handlers/mod.rs      # add fallback + CORS changes
        ├── state.rs             # add Web Push deps
        └── services/
            └── web_push.rs       # NEW — push notification logic
```

## 4. Platform Abstraction

The desktop client currently imports Tauri APIs directly in three hooks:

- `desktop/src/hooks/useTray.ts` — `invoke('set_unread_count')`
- `desktop/src/hooks/useNotifications.ts` — `@tauri-apps/plugin-notification`
- `desktop/src/hooks/useDeepLink.ts` — `@tauri-apps/plugin-deep-link`
- File dialogs in the composer use `@tauri-apps/plugin-dialog`.

These hooks are called from `desktop/src/App.tsx` via `AuthenticatedShell`.

### 4.1 Platform Contract

Create `desktop/src/platform/index.ts`:

```typescript
export interface Platform {
  useTray: (options: { unreadCount: number; enabled: boolean }) => void;
  useDeepLink: () => void;
  useNotifications: (options: {
    userId: string;
    enabled: boolean;
  }) => {
    maybeNotify: (event: ServerEvent) => Promise<void>;
    request: () => Promise<void>;
  };
  FilePicker: React.ComponentType<FilePickerProps> | undefined;
}
```

### 4.2 Desktop Implementation

`desktop/src/platform/desktop.ts` re-exports the current Tauri-based hooks and
a Tauri file-picker wrapper unchanged.

### 4.3 Web Implementation

`desktop/src/platform/web.ts` provides:

- `useTray`: no-op (browsers have no system tray API).
- `useDeepLink`: no-op (URL routing is handled by React Router).
- `useNotifications`: Web Push-based implementation. It requests notification
  permission, subscribes the service worker to the server, and calls
  `navigator.serviceWorker.ready` + `swRegistration.showNotification(...)` to
  display notifications.
- `FilePicker`: a small wrapper around `<input type="file">` that matches the
  Tauri dialog return shape so the composer can stay platform-agnostic.

### 4.4 Shared App Entry Refactor

Move the provider tree and hook wiring out of `desktop/src/App.tsx` into a new
`desktop/src/PlatformShell.tsx`. `PlatformShell` accepts a `Platform` object
as a prop and renders the shared `<Shell />`.

`desktop/src/App.tsx` and `web/src/App.tsx` both import `PlatformShell` and pass
their respective platform objects.

This keeps changes surgical: the desktop entry point shrinks, the web entry
point is small, and the shared UI code does not change.

## 5. Web Build (`web/`)

`web/` is a standard Vite React package that mirrors `desktop/` but has no
Tauri dependency.

### 5.1 package.json

- Same React, React Router, Tailwind, TypeScript, and Vite versions as
  `desktop/package.json`.
- No `@tauri-apps/*` packages.
- Adds `vite-plugin-pwa` (or `workbox-window`) for service worker generation.

### 5.2 vite.config.ts

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    VitePWA({
      registerType: 'autoUpdate',
      manifest: false, // use public/manifest.json
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
      },
    }),
  ],
  server: {
    port: 5174,
    proxy: {
      '/api': 'http://localhost:3000',
      '/websocket': {
        target: 'ws://localhost:3000',
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
  },
});
```

The dev proxy keeps CORS out of the local dev loop; CORS changes below are for
production deployments where the Web UI is served from a different host.

### 5.3 tsconfig.json

Extends `desktop/tsconfig.json` and adds `desktop/src` to `include` so the web
build can import shared code via relative paths (`../../desktop/src/...`).

## 6. Server-Side Static Asset Serving

### 6.1 Configuration

Add a `web` section to `ruckchat.yaml`:

```yaml
web:
  enabled: true
  # Optional: serve assets from this directory instead of embedded assets.
  # If omitted, the server uses assets embedded at compile time.
  path: "/usr/share/ruckchat/web"
```

Add `WebConfig` to `crates/ruckchat-config/src/lib.rs`:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub path: Option<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
        }
    }
}
```

### 6.2 Embedded Assets

Use the `include_dir` crate in `server/Cargo.toml`. At compile time embed
`web/dist`:

```rust
use include_dir::{Dir, include_dir};

static WEB_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../web/dist");
```

The release build workflow becomes:

```bash
cd web && pnpm install && pnpm build
cd ..
cargo build --release -p ruckchat-server
```

For development and custom branding, operators set `web.path` and skip the
embedded assets.

### 6.3 Handler

Create `server/src/handlers/web_assets.rs`. It serves:

- Exact file matches from the configured directory or embedded directory.
- Any unmatched path falls back to `index.html` so React Router handles
  client-side routes.

In `server/src/handlers/mod.rs`, add `.fallback_service(web_assets::service(state))`
only when `config.web.enabled`. Because fallback only runs when no defined API
route matches, REST 404s are unaffected.

## 7. CORS and Cookie Authentication

The server currently layers `CorsLayer::permissive()`. That sends
`Access-Control-Allow-Origin: *`, which browsers reject for credentialed requests.
Because the Web UI uses `credentials: 'include'` (already set in
`desktop/src/api/client.ts`), the CORS layer must be fixed.

### 7.1 New CORS Layer

Replace `CorsLayer::permissive()` with an explicit layer that:

- Reflects the caller's `Origin` when it matches an allowed origin.
- Sets `Access-Control-Allow-Credentials: true`.
- Allows methods: GET, POST, PATCH, DELETE.
- Allows headers: `Content-Type`, `Authorization`, and any headers used by
  `rmcp` / MCP.

Add `web.allowed_origins` to `ruckchat.yaml`:

```yaml
web:
  enabled: true
  allowed_origins:
    - "https://chat.example.com"
```

If `allowed_origins` is empty or omitted, the server computes the allowed set as
just the origin implied by `base_url` (same-origin deployments). This preserves
security for the default single-host deployment while still fixing the
`Allow-Credentials` issue.

### 7.2 SameSite Cookie Considerations

The current session cookie is `SameSite=Strict; Path=/`. This works when the Web
UI is served from the same origin as the API (the default deployment model).

If an operator later hosts the Web UI on a different origin, `SameSite=Strict`
cookies will not be sent cross-origin. Supporting that requires HTTPS,
`SameSite=None; Secure`, and an explicitly allowed origin. We can defer that
sub-mode until it is requested; the design keeps the option open by making the
cookie `SameSite` derive from `config.environment` or a future
`web.cross_origin` flag.

## 8. Web Push Notifications

### 8.1 VAPID Keys

Add to `ruckchat.yaml`:

```yaml
web_push:
  enabled: true
  subject: "mailto:admin@example.com"
  vapid_public_key: "BASE64URL_PUBLIC_KEY"
  vapid_private_key: "BASE64URL_PRIVATE_KEY"
```

The `vapid_public_key` is exposed to clients so the service worker can subscribe.
The private key is used by the server to sign push messages.

### 8.2 Database Schema

Add a migration for `web_push_subscriptions`:

```sql
CREATE TABLE web_push_subscriptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint        TEXT NOT NULL,
    p256dh          TEXT NOT NULL,
    auth            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, endpoint)
);

CREATE INDEX idx_web_push_subscriptions_user_id ON web_push_subscriptions(user_id);
```

### 8.3 Domain Model and Repository

Add `WebPushSubscription` to `ruckchat-domain` and a `WebPushSubscriptionRepository`
trait. Add a SQLx implementation in `server/src/repositories/web_push.rs`.

### 8.4 REST Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/web-push/vapid-key` | Returns the VAPID public key |
| POST | `/web-push/subscribe` | Stores a PushSubscription |
| POST | `/web-push/unsubscribe` | Removes a PushSubscription |

Request/response DTOs live in `server/src/handlers/dto.rs`.

### 8.5 Service

Create `server/src/services/web_push.rs`:

- `subscribe(user_id, subscription)` — insert or update.
- `unsubscribe(user_id, endpoint)` — delete.
- `notify(user_id, title, body)` — fetch the user's subscriptions and send a
  signed push message via `web-push` crate (or equivalent). Remove stale
  endpoints (HTTP 410 / 404) on failure.

Hook the service into the existing event bus: when a `message.created` event is
emitted and the target user is offline (or regardless, depending on settings),
call `web_push.notify(...)` for DMs and `@mentions`.

### 8.6 Web Client Service Worker

- `web/public/sw.js` listens for `push` events and calls
  `self.registration.showNotification(title, { body, icon, data })`.
- `web/src/service-worker/register.ts` registers the SW, requests notification
  permission, fetches the VAPID key from `/web-push/vapid-key`, and calls
  `pushManager.subscribe(...)`.
- The resulting `PushSubscription` JSON is POSTed to `/web-push/subscribe`.

## 9. Responsive Layout

The shared Tailwind classes in `desktop/src/components/` should be reviewed for
hard-coded widths (e.g., fixed sidebar widths). Add responsive variants:

- On small screens, the sidebar collapses to a hamburger/drawer.
- The composer and message pane use full width.
- Touch targets are at least `44px`.

This is primarily CSS/Tailwind work; no server changes are required.

## 10. File Attachments in the Browser

Desktop uses Tauri file dialogs. The composer must switch to the web file
picker on the Web UI.

- Add a `FilePicker` component to the platform contract (see §4).
- In `desktop/src/platform/desktop.ts`, `FilePicker` wraps the Tauri dialog and
  returns a `File` object the composer can pass to the API.
- In `desktop/src/platform/web.ts`, `FilePicker` renders `<input type="file">`
  and returns the selected `File` object.
- The existing `FilesApi` and attachment flow remain the same; the composer
  only changes which picker component it renders.

The shared `ApiClient` currently sends JSON bodies. Multipart upload support must
be added for `POST /files`. Add a `uploadFile` helper in `desktop/src/api/files.ts`
(or a new method on `ApiClient`) that uses `FormData` and `fetch` with
`credentials: 'include'`.

## 11. API Specification Additions

Update `server/openapi.yaml` with the new endpoints:

- `GET /web-push/vapid-key`
- `POST /web-push/subscribe`
- `POST /web-push/unsubscribe`

No changes to existing endpoints are required.

## 12. Testing Strategy

- **Unit tests**: web platform shims for `useTray`, `useDeepLink`, and
  `useNotifications` in `web/src/platform/`.
- **Integration tests**: server tests for subscribe/unsubscribe endpoints using a
  PostgreSQL database.
- **Web Push**: a unit test for `WebPushService` using a mock HTTP push server.
- **Static assets**: integration test verifying the fallback to `index.html` and
  serving of a known asset.
- **Desktop regression**: existing desktop tests must continue to pass after the
  platform abstraction refactor.

## 13. Implementation Order

1. **Platform abstraction**
   - Create `desktop/src/platform/` contract and desktop implementation.
   - Refactor `desktop/src/App.tsx` to use `PlatformShell`.
   - Verify desktop still builds and tests pass.

2. **Web package scaffold**
   - Create `web/` with Vite, React, Tailwind, and PWA config.
   - Add web platform implementations.
   - Create `web/src/App.tsx` that renders `PlatformShell`.

3. **Server static asset serving**
   - Add `WebConfig` to `ruckchat-config`.
   - Add `include_dir` and `web_assets` handler to server.
   - Wire fallback service into the router.

4. **CORS fix**
   - Replace `CorsLayer::permissive()` with explicit credentials-aware CORS.
   - Add `web.allowed_origins` to config.

5. **Web Push**
   - Add migration, domain model, repository, service, and handlers.
   - Add VAPID config to `ruckchat.yaml`.
   - Integrate `WebPushService` into the event bus.

6. **PWA polish**
   - Add manifest, icons, service worker, and registration.
   - Offline caching of static assets and pending message queue.

7. **Responsive layout and file picker**
   - Add mobile breakpoints to shared components.
   - Add platform `FilePicker` and multipart upload support.

8. **Docs and tests**
   - Update `server/openapi.yaml`.
   - Write `docs/ADR-011-Web-UI.md`.
   - Update `README.md`, `CLAUDE.md`, and `docs/IMPLEMENTATION_PLAN.md`.
   - Add unit and integration tests.

## 14. Risks and Tradeoffs

| Risk | Mitigation |
|------|-----------|
| Sharing code with desktop creates coupling | Keep platform-specific code isolated in `desktop/src/platform/`. |
| Embedded assets increase server binary size | `web.path` lets operators opt out of embedded assets. |
| Web Push key rotation is hard | Store per-user subscriptions; generate a new VAPID key pair and let clients re-subscribe. |
| SameSite=Strict blocks cross-origin cookie auth | Default deployment serves Web UI from the same origin. Document the HTTPS/SameSite=None path for separate hosting. |
| CORS changes affect MCP clients | CORS layer applies globally; test MCP integration tests after the change. |

## 15. Open Decisions

The following decisions are captured in the design but can be revisited during
implementation:

1. Whether to use `vite-plugin-pwa` or a hand-written service worker.
2. Whether to embed `web/dist` at compile time or only support `web.path` in
the first iteration.
3. Whether to send Web Push notifications for *all* messages or only
mentions/DMs.
4. Whether to add a `web.cross_origin` flag to switch cookies to
`SameSite=None; Secure`.
