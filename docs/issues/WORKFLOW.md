# Issue Resolution Workflow

This document defines the implementation workflow for the open issues tracked in
`docs/issues/ISSUES{0-9}.md`. The workflow groups related issues into phases so
that foundation work is completed before higher-level features are built on top
of it.

## Current status

- **Phase 1 — Foundation** ✅ Complete (commit `2279700`).
  - ISSUES1 — Light/dark theme system implemented.
  - ISSUES9 — `allow_registration` site setting implemented.
- **Phase 2 — Composer and Message Format** 🚧 In progress.
  - ISSUES0 — @mentions support: complete (commit `ba9ca30`).
  - ISSUES2 — Tiptap composer is in place; `@farscrl/tiptap-extension-spellchecker`
    integration with a server-side Hunspell API is the remaining work.
- **Phase 3 — Conversation Discovery** ⏸ Pending Phase 2.
  - ISSUES3 — Single-organization auto-redirect.
  - ISSUES4 — Channel creation and management UI.
  - ISSUES5 — Complete direct message functionality.
- **Phase 4 — Admin UI Polish** ⏸ Pending Phase 2.
  - ISSUES6 — Back-to-chat link in admin UIs.
  - ISSUES7 — Complete organization admin UI.
  - ISSUES8 — User editor modal in server admin.

## Guiding principles

- Follow the RuckChat implementation loop: Read docs → Plan → Write code →
  `cargo fmt` → `cargo check` → `cargo clippy` → `cargo nextest` → Fix → Update
  docs → Commit → Update codegraph.
- Apply the `ponytail` skill: prefer deletion, reuse existing code, use
  stdlib/native/installed dependencies, question speculative features.
- Every backend change needs an OpenAPI update, integration tests, and ADR
  updates when architecture changes.
- Every frontend change needs a type check, unit tests, and PWA/desktop parity
  verification.
- No AI attribution in commits, code, or documentation.

## Phase grouping

| Phase | Issues | Theme | Why grouped |
|-------|--------|-------|-------------|
| 1 | ISSUES1, ISSUES9 | Foundation | Shared tokens + server settings affect all later UI and auth behavior. |
| 2 | ISSUES0, ISSUES2 | Composer / message format | Mentions and Tiptap both change how messages are authored, stored, and rendered. |
| 3 | ISSUES3, ISSUES4, ISSUES5 | Conversation discovery | Redirects, channel CRUD, and DMs all touch routing, sidebar, and conversation APIs. |
| 4 | ISSUES6, ISSUES7, ISSUES8 | Admin UI polish | Back links, complete org admin, and user editor modal share the admin shell components. |

---

## Phase 1 — Foundation

### Issues

- [ISSUES1](ISSUES1.md) — Light theme with light/dark toggle.
- [ISSUES9](ISSUES9.md) — Site setting to allow/deny user registrations.

### Goals

1. Establish a theme-token system that all later UI work can rely on.
2. Add a server-wide `allow_registration` setting and enforce it.

### Order of work

1. **Theme tokens (ISSUES1)**
   - Audit current hardcoded colors in `desktop/src/components/**/*.tsx`.
   - Define CSS custom properties for background, surface, text, border, accent,
     and danger colors in a shared CSS file imported by both `desktop` and `web`.
   - Configure Tailwind to read the custom properties and add `dark:` variants.
   - Add `theme` (`light` | `dark` | `system`) to `desktop/src/hooks/useSettings.ts`
     and persist it in `localStorage`.
   - Add a theme toggle to `desktop/src/components/Settings.tsx`.
   - Apply tokens to all shared components in `desktop/src/components/` so the
     UI works in both themes.
   - Update `web/public/manifest.json` and `web/index.html` `theme-color` to
     respect the active theme.
   - Verify with `cd desktop && pnpm typecheck` and `cd web && pnpm build`.

2. **Registration gate (ISSUES9)**
   - Add `allow_registration: boolean` (default `true`) to
     `server/openapi.yaml` `ServerSettings` and `UpdateServerSettingsRequest`.
   - Add a database migration for `server_settings.allow_registration`.
   - Update `server/src/services/server_settings.rs` to load and merge the
     setting, with YAML override precedence.
   - Update `server/src/config.rs` to expose an optional YAML override.
   - Enforce the gate in `server/src/handlers/auth.rs` (before auth service)
     returning `403 Forbidden` when disabled.
   - Add checkbox to `desktop/src/components/admin/ServerAdminSettings.tsx`.
   - Hide/disable register tab in `desktop/src/components/AuthScreen.tsx` when
     the setting is `false`.
   - Add backend integration tests for allowed/blocked registration.
   - Verify with `cargo fmt`, `cargo check`, `cargo clippy`,
     `cargo nextest run --workspace`, and `cd desktop && pnpm typecheck`.

### Cross-phase impact

- Theme tokens are consumed by every later phase.
- `allow_registration` can be toggled before testing Phase 4 user creation.

### Verification

- `cargo fmt --all` passes.
- `cargo check --workspace` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo nextest run --workspace` passes (223 tests).
- `cd desktop && pnpm typecheck && pnpm test` passes (25 tests).
- `cd web && pnpm build` succeeds.
- `codegraph index` refreshed.
- Commit: `2279700` — "Add light/dark theme and user registration gate setting".
- Manual check: theme toggle works; registering a new user is blocked when the
  setting is off.

---

## Phase 2 — Composer and Message Format

### Issues

- [ISSUES0](ISSUES0.md) — @mentions support.
- [ISSUES2](ISSUES2.md) — WYSIWYG Tiptap composer with spell check.

### Goals

1. Replace the plain-text composer with a Tiptap editor that stores ProseMirror
   JSON.
2. Implement `@display_name` mentions as first-class nodes.
3. Store `mentioned_user_ids` on the message row and emit mention notifications.

### Order of work

1. **Backend: message format and mentions (ISSUES0)**
   - Extend `server/openapi.yaml` `Message` schema with `mentioned_user_ids`.
   - Add `mentioned_user_ids` column to the `messages` table via migration.
   - Update `crates/ruckchat-domain/src/message.rs` domain model.
   - Update `server/src/repositories/message.rs` to read/write the column.
   - Update `server/src/services/message.rs` to parse `@display_name` tokens in
     Tiptap content, resolve them to user IDs, and store the resolved set.
   - Emit a real-time `mention` event via the WebSocket event bus for each
     mentioned user.
   - Ensure mention extraction works for channel messages, thread replies, and
     DM messages.

2. **Frontend: Tiptap composer (ISSUES2)**
   - Add Tiptap dependencies to `desktop/package.json` and `web/package.json`:
     `@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/extension-mention`, and
     the spell-check extension.
   - Create a new `desktop/src/components/TiptapComposer.tsx` that replaces the
     `textarea` in `Composer.tsx`.
   - Implement mention suggestions by display name/email using an existing or
     new user-search API.
   - Store the editor output as ProseMirror JSON; send it as the message
     `content`.
   - Remove the Markdown preview toggle and the `showPreview` state.
   - Preserve `Enter` to send and `Shift+Enter` for newlines.
   - Integrate the spell-check extension with a dictionary/backend endpoint.

3. **Frontend: message rendering (ISSUES0 / ISSUES2)**
   - Update `desktop/src/components/MessageItem.tsx` to render ProseMirror JSON,
     including mention nodes as styled, clickable tokens.
   - Update `desktop/src/components/ThreadPane.tsx` for the same renderer.

### Cross-phase impact

- ProseMirror JSON becomes the canonical message format; Phase 3 and Phase 4
  features must use it.
- Mention notifications may influence unread badge logic; coordinate with
  `desktop/src/hooks/useUnread.ts`.

### Verification

- All backend checks pass (`cargo fmt`, `cargo check`, `cargo clippy`,
  `cargo nextest`).
- Frontend type checks and unit tests pass.
- Manual check: type `@` in the composer, select a user by display name, send,
   and see the rendered mention; the mentioned user receives a notification.

---

## Phase 3 — Conversation Discovery

### Issues

- [ISSUES3](ISSUES3.md) — Single-organization auto-redirect to #general.
- [ISSUES4](ISSUES4.md) — Channel creation and management UI.
- [ISSUES5](ISSUES5.md) — Complete direct message functionality.

### Goals

1. Redirect single-org users to the right channel automatically.
2. Let users create, update, archive, and manage channels and private-channel
   membership.
3. Provide a complete DM list and start-DM experience.

### Order of work

1. **Backend policy alignment (ISSUES4)**
   - Verify `server/src/services/channel.rs` and `AuthorizationService`: channel
     creation must allow any organization member per the recorded decision.
   - Update the service test that currently forbids member-created channels.
   - Ensure private-channel invite endpoints exist (list org members, add/remove
     channel members). Add to OpenAPI if missing.

2. **Single-org redirect (ISSUES3)**
   - Update `desktop/src/PlatformShell.tsx` (or a new `/org` route component) to
     redirect to the last selected channel when available, otherwise to the
     organization's `general` channel, when the user belongs to exactly one
     organization.
   - Persist the last selected channel in `localStorage`.

3. **Channel CRUD UI (ISSUES4)**
   - Add a "+" button next to the Channels section in
     `desktop/src/components/Sidebar.tsx`.
   - Create `desktop/src/components/CreateChannelModal.tsx` with name,
     public/private toggle, topic, purpose, and optional initial member invites.
   - Add channel context-menu actions: edit topic/purpose, archive, unarchive.
   - Render public, private, and archived channels in the sidebar; archived
     channels in a collapsed section.
   - Wire `desktop/src/api/channels.ts` to create/update/archive and
     manage private-channel membership.

4. **DM UI (ISSUES5)**
   - Add a "Direct messages" section in `desktop/src/components/Sidebar.tsx`
     with a "New message" button.
   - Create `desktop/src/components/StartDmModal.tsx` for searching members and
     creating a multi-user DM.
   - Render DM conversations by combined member display names; allow
     hide/archive from the current user's sidebar.
   - Verify `desktop/src/hooks/useDirectMessages.ts` and
     `desktop/src/api/directMessages.ts` support list/start operations.
   - Ensure thread replies work for DM conversations.

### Cross-phase impact

- Channel and DM selection updates must write to the "last selected channel"
  store used in ISSUES3.
- The new theme tokens from Phase 1 apply to all new modals and sidebar updates.

### Verification

- Backend integration tests for channel creation, archive, and member management
  pass.
- Frontend type checks and tests pass.
- Manual check: login with one org lands on `general`; create a private channel,
  invite a member, archive it; start a group DM and send messages.

---

## Phase 4 — Admin UI Polish

### Issues

- [ISSUES6](ISSUES6.md) — Back-to-chat link in admin UIs.
- [ISSUES7](ISSUES7.md) — Complete organization admin UI.
- [ISSUES8](ISSUES8.md) — User editor modal in server admin.

### Goals

1. Add consistent back-to-chat links in server and org admin shells.
2. Finish all org admin screens.
3. Replace inline user editing with a full user modal that supports create and
   edit, plus destructive actions.

### Order of work

1. **Back-to-chat links (ISSUES6)**
   - Add a top-right "Back" `NavLink` to
     `desktop/src/components/admin/ServerAdminShell.tsx` and
     `desktop/src/components/admin/OrgAdminShell.tsx`, matching the
     `Settings.tsx` style.
   - Link destination: the most recently active channel from `localStorage` or
     router history; fall back to `/`.

2. **Complete org admin (ISSUES7)**
   - `OrgAdminMembers.tsx`: invite by email, list members, remove members,
     change member roles.
   - `OrgAdminRoles.tsx`: create, list, edit, delete custom roles and assign
     permissions.
   - `OrgAdminPermissions.tsx`: create, list, edit, delete custom permissions.
   - `OrgAdminEmoji.tsx`: upload/list custom emoji (delete optional).
   - `OrgAdminTeams.tsx`: create/list teams, add/remove members, assign team
     rooms.
   - `OrgAdminShell.tsx`: make navigation collapsible on small screens like the
     main shell.
   - Ensure `desktop/src/api/orgAdmin.ts` and backend handlers support all
     required operations; update OpenAPI as needed.

3. **Server admin user modal (ISSUES8)**
   - Create `desktop/src/components/admin/EditUserModal.tsx` usable for both
     creating and editing users.
   - Editable fields: display name, email, avatar URL.
   - Actions: toggle `is_server_admin`, reset password with generated password
     shown, deactivate/reactivate account, delete user (with confirmation).
   - Place destructive actions in a "Danger zone" with confirmation dialogs.
   - Update `desktop/src/components/admin/ServerAdminUsers.tsx` to trigger the
     modal instead of inline editing.
   - Verify backend endpoints in `server/src/handlers/server_admin.rs` and
     `server/src/services/server_admin.rs` cover all actions.

### Cross-phase impact

- Org admin and user editor modals use the theme tokens from Phase 1.
- The back-to-chat link destination depends on the last-selected channel store
  introduced in Phase 3.

### Verification

- Backend checks and integration tests pass.
- Frontend type checks and tests pass.
- Manual check: server admin users can be created and edited in a modal; org
  admin can manage members/roles/permissions/emoji/teams; admin back links
  return to chat.

---

## After all phases

1. Update `server/openapi.yaml` and regenerate `desktop/src/api/schema.ts` if
   any schemas changed.
2. Update `book/*.md` and relevant `docs/ADR-*.md` if architecture changed
   (notably the ProseMirror JSON message format and theme token system).
3. Update root `CLAUDE.md` if new commands or conventions were introduced.
4. Run the full implementation loop one final time.
5. Commit as `Brian Tafoya <btafoya@briantafoya.com>` with no AI attribution.
6. Run `codegraph index` to refresh the structural index.

## Risks and notes

- **ISSUES0 + ISSUES2 coupling**: switching to ProseMirror JSON is the larger
  architectural change. If it proves too disruptive, fall back to storing
  Markdown plain text with mention metadata, but keep the decision to use
  `@display_name`.
- **Backend authorization for channel creation**: the existing service test
  asserts members cannot create channels. The recorded decision says any member
  can create them; the policy and tests must be aligned.
- **Tiptap spell-check extension**: the requested extension may require a
  dictionary backend or local dictionary file. If it is unavailable or
  unmaintained, fall back to browser spell-check and document the change.
- **Admin UI scope**: ISSUES7 is the broadest frontend task. Consider splitting
  it into per-screen PRs while keeping the overall workflow intact.
