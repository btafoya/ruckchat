# Phase 1 — Foundation Task Breakdown

This document is the output of `/sc:spawn phase 1`. It decomposes the Phase 1
foundation work from `WORKFLOW.md` into executable tasks with dependencies and
delegation assignments. No implementation code is written here; this is a plan
for execution via `/sc:implement`, `/sc:design`, and `/sc:test`.

## Epic

**Phase 1 — Foundation**

Implement ISSUES1 (light/dark theme tokens and toggle) and ISSUES9
(`allow_registration` site setting with backend enforcement and UI).

## Stories and tasks

### Story A — Theme system (ISSUES1)

| Task | ID | Description | Delegated to | Blocked by |
|------|-----|-------------|--------------|------------|
| A.1 Theme token audit and CSS variables | P1.1 | Audit hardcoded colors in `desktop/src/components/**/*.tsx`, define semantic CSS custom properties, and configure Tailwind `dark:` variants. | `/sc:design` then `/sc:implement` | — |
| A.2 Theme state and Settings toggle | P1.2 | Add `theme` (`light`/`dark`/`system`) to `useSettings.ts`, persist in `localStorage`, default to system preference, add toggle to `Settings.tsx`, and update root class / PWA meta. | `/sc:implement` | A.1 |
| A.3 Apply theme tokens to all shared components | P1.3 | Replace hardcoded colors in shared components with theme tokens for full light/dark support. | `/sc:implement` | A.1 |

### Story B — Registration gate (ISSUES9)

| Task | ID | Description | Delegated to | Blocked by |
|------|-----|-------------|--------------|------------|
| B.1 `allow_registration` backend schema and migration | P1.4 | Add `allow_registration` to `server_settings`, domain model, repository, service, config override, and OpenAPI. | `/sc:implement` | — |
| B.2 Enforce registration gate in auth handler | P1.5 | Return `403 Forbidden` from the register handler when `allow_registration` is false; add integration tests. | `/sc:implement` | B.1 |
| B.3 Server admin and AuthScreen UI | P1.6 | Add checkbox to `ServerAdminSettings.tsx`; hide/disable register UI in `AuthScreen.tsx` when disabled. | `/sc:implement` | B.1, B.2 |

### Story C — Verification and closure

| Task | ID | Description | Delegated to | Blocked by |
|------|-----|-------------|--------------|------------|
| C.1 Phase 1 verification and commit | P1.7 | Run `cargo fmt/check/clippy/nextest`, desktop and web type checks/tests, update docs, commit, refresh codegraph. | `/sc:test` | A.2, A.3, B.3 |

## Task dependency graph

```text
P1.1 ──┬── P1.2 ──┐
       └── P1.3 ──┤
                  └─ P1.7
P1.4 ── P1.5 ── P1.6 ──┘
```

## Execution strategy

- **Independent tracks run in parallel**:
  - Theme track: P1.1 → (P1.2 + P1.3) → P1.7
  - Registration track: P1.4 → P1.5 → P1.6 → P1.7
- **Verification gate**: P1.7 must wait for both tracks to complete.

## Quality gates per task

- Rust changes: `cargo fmt --all`, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo nextest run --workspace`.
- TypeScript changes: `cd desktop && pnpm typecheck && pnpm test` and
  `cd web && pnpm build`.
- Documentation: update `server/openapi.yaml`, regenerate
  `desktop/src/api/schema.ts` if schemas changed, and update `CLAUDE.md` /
  `book/*.md` / `docs/ADR-*.md` if architecture changed.

## Risk notes

- P1.1 is the most invasive because it touches every shared component. If the
  token naming is wrong, P1.3 will need rework. Finalize token names in P1.1
  before starting P1.3.
- P1.5 should only gate public self-registration. Server-admin user creation
  (P1.6 / ISSUES8) and organization invitations must remain possible when the
  setting is false.
- P1.6 depends on the current server settings UI structure; verify
  `ServerAdminSettings.tsx` reads/writes `ServerSettings` before adding the
  checkbox.

## Next commands

Execute the tasks in dependency order, e.g.:

```
/sc:implement P1.1
/sc:implement P1.4
/sc:implement P1.2
/sc:implement P1.3
/sc:implement P1.5
/sc:implement P1.6
/sc:test P1.7
```
