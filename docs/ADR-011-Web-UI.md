# ADR-011: Web UI and Browser Client

## Status

Proposed — pending implementation in Phase 10.

## Context

RuckChat has a Tauri + React desktop client that provides the full messaging
experience. Small teams increasingly expect to use a self-hosted chat tool from
any device without installing a native application. A browser-based Web UI lets
users join conversations immediately and also enables Progressive Web App
(PWA) features such as install prompts, offline caching, and push notifications.

We needed to decide:

- Whether to build a separate web codebase or reuse the desktop React code.
- Where the Web UI lives in the repository and deployment model.
- How the server delivers static assets.
- How browser-specific capabilities (notifications, file picker, deep links) are
  handled without duplicating the shared UI.
- How Web Push notifications and cross-origin cookie authentication work.

## Decision

We will add a browser-based Web UI as **Phase 10** in the implementation order.

- **Code sharing**: The Web UI reuses the existing React components, hooks, API
  client, and contexts in `desktop/src`. A small `desktop/src/platform/`
  abstraction layer provides desktop (Tauri) and web (browser) implementations
  of native-only features.
- **Location**: A new top-level `web/` directory contains the Vite React build,
  its own `package.json`, and web-specific entry points. It imports shared code
  from `desktop/src` via relative paths.
- **Static asset serving**: The Rust server serves the Web UI as static assets.
  `ruckchat.yaml` gains a `web` section with `enabled` and an optional `path`.
  When `path` is omitted, the server serves assets embedded at compile time via
  `include_dir`.
- **PWA**: The web build includes a service worker, manifest, and install
  prompt. Offline resilience is limited to asset caching and queued outgoing
  messages in the first iteration.
- **Web Push**: The server manages a VAPID key pair stored in
  `ruckchat.yaml`. Clients subscribe via `POST /web-push/subscribe` and receive
  push notifications for direct messages and `@mentions`.
- **CORS**: The current `CorsLayer::permissive()` is replaced with an explicit
  credentials-aware CORS configuration so the Web UI can use the existing
  HTTP-only `ruckchat_session` cookie when served from a different origin.
- **Cookie policy**: The default deployment serves the Web UI from the same
  origin as the API, keeping `SameSite=Strict` cookies working. A future
  `web.cross_origin` flag can switch to `SameSite=None; Secure` for separate
  hosting.

## Consequences

### Positive

- Minimal duplication: one React codebase powers both desktop and web clients.
- Self-hosted deployments can be a single binary that serves its own UI.
- Browser users get install prompts and push notifications comparable to the
  desktop client.
- The platform abstraction keeps Tauri-specific APIs out of shared components.

### Negative

- `desktop/src` is now a shared library in addition to being the desktop source.
  Its name is slightly misleading; renaming it is deferred to avoid a large
  refactor in this phase.
- Embedded assets increase the server binary size and add a build dependency on
  `pnpm build` in `web/`.
- Web Push requires operators to generate and secure a VAPID key pair.
- The CORS change touches all cross-origin requests, including MCP clients, and
  must be tested carefully.

## Related

- `docs/design/WEB-UI-DESIGN.md`
- `book/007-Desktop.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/ADR-010-Runtime-YAML-Configuration.md`
