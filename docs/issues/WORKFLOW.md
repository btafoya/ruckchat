# Issue Resolution Workflow

This document defines the implementation workflow for the open issues tracked in
`docs/issues/ISSUES{0-21}.md`. The workflow groups related issues into phases so
that foundation work is completed before higher-level features are built on top
of it.

## Current status

- **Phase 1 — Foundation** ✅ Complete (commit `2279700`).
  - ISSUES1 — Light/dark theme system implemented.
  - ISSUES9 — `allow_registration` site setting implemented.
- **Phase 2 — Composer and Message Format** ✅ Complete.
  - ISSUES0 — @mentions support: complete (commit `ba9ca30`).
  - ISSUES2 — Tiptap composer complete; `@farscrl/tiptap-extension-spellchecker`
    integration with the embedded `ruckchat-spelling` Hunspell API is complete.
- **Phase 3 — Conversation Discovery** ✅ Complete (commit `51c5372`).
  - ISSUES3 — Single-organization auto-redirect to the last-selected channel
    (or `#general`) implemented via `OrgIndexRoute` / `ChannelIndexRoute` and
    `desktop/src/lastConversation.ts`.
  - ISSUES4 — Channel creation and management UI complete: any organization
    member can create channels; the creator or an organization manager can
    edit/archive/unarchive and manage private-channel membership; any member
    can self-join a public channel (auto-join on first post).
  - ISSUES5 — Direct message functionality complete: start (with reuse of an
    existing conversation), list with resolved display names, hide/reappear
    on new message, all wired through `desktop/src/components/StartDmModal.tsx`
    and `Sidebar.tsx`.
- **Phase 4 — Admin UI Polish** ✅ Complete.
  - ISSUES6 — Back-to-chat link added to `ServerAdminShell.tsx` (to `/`) and
    `OrgAdminShell.tsx` (to the last-selected channel/DM via
    `lastConversation.ts`, falling back to the organization's channel index).
  - ISSUES7 — Organization admin UI completed: `OrgAdminMembers.tsx` now
    supports invite-by-email, role change, and removal (backed by the
    existing `/organizations/{id}/members` endpoints); `OrgAdminTeams.tsx`
    gained per-team member and room management panels backed by new
    `TeamMembershipRepository`/`TeamRoomRepository` `delete` methods and new
    `/api/v1/admin/organizations/{id}/teams/{team_id}/members[/…]` and
    `/rooms[/…]` endpoints; `OrgAdminEmoji.tsx` now uploads a file via the
    files API instead of accepting a raw file ID; `OrgAdminShell.tsx` gained a
    collapsible mobile navigation matching the main `Shell`/`Sidebar`
    pattern. Roles, Permissions, and organization Settings screens were
    already complete from prior work.
  - ISSUES8 — `EditUserModal.tsx` replaces inline row editing in
    `ServerAdminUsers.tsx`: profile fields, server-admin promote/demote,
    password reset, deactivate/reactivate, and a danger-zone permanent delete
    with confirmation. Deletion is backed by a new
    `DELETE /api/v1/server/users/{user_id}` endpoint and
    `UserRepository::delete`; deleting a user with existing message history
    or organization ownership returns `409 Conflict` (foreign-key
    violations are mapped to a clear error) and the last server admin cannot
    be deleted or demoted.
- **Phase 5 — Shell and Navigation** ⏳ Open.
  - ISSUES12 — Collapsible sidebar (full ↔ narrow).
  - ISSUES13 — Sidebar on mobile.
  - ISSUES14 — Admin menu icons in top bar.
- **Phase 6 — Home and User Profile** ⏳ Open.
  - ISSUES15 — Single-organization home redirect.
  - ISSUES16 — Organization home unread-messages view.
  - ISSUES17 — Server-stored theme preference.
- **Phase 7 — Messages and Composer** ⏳ Open.
  - ISSUES10 — Submitted message duplicated.
  - ISSUES18 — Add message delete option.
  - ISSUES19 — Remove Markdown preview from composer.
- **Phase 8 — Direct Messages and Server Admin Completion** ⏳ Open.
  - ISSUES20 — Finish direct messages modal (empty user list).
  - ISSUES21 — Finish CRUD for server admins view.
- **ISSUES11 — Environment/Tooling Note** ⏳ Open.
  - `tokenjuice wrap` shell wrapper observed during an agent session; not a RuckChat product change.

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
| 5 | ISSUES12, ISSUES13, ISSUES14 | Shell and navigation | Sidebar collapse, mobile sidebar, and admin top-bar icons all reshape the main shell. |
| 6 | ISSUES15, ISSUES16, ISSUES17 | Home and user profile | Single-org redirect, unread home view, and server-stored theme are user-facing landing/profile concerns. |
| 7 | ISSUES10, ISSUES18, ISSUES19 | Messages and composer | Send duplication, message deletion, and removing the preview toggle are all message-surface work. |
| 8 | ISSUES20, ISSUES21 | DM and server admin completion | The DM start modal and server admins list both finish partially built Phase 4/Phase 3 features. |

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

- `cargo fmt --all` passes.
- `cargo check --workspace --all-targets` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo nextest run --workspace` passes (242 tests), including
  `server/tests/spelling.rs` and the updated `server_admin.rs` /
  `auth.rs` settings-update tests.
- `cd desktop && pnpm typecheck && pnpm test` passes (25 tests).
- `cd web && pnpm typecheck && pnpm build` succeeds.
- Manual check: type `@` in the composer, select a user by display name, send,
   and see the rendered mention; the mentioned user receives a notification.
- Manual check: type a misspelled word in the composer and see the wavy
  underline and suggestion popup from the spell-checker.

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

- `cargo fmt --all` passes.
- `cargo check --workspace` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo nextest run --workspace` passes (255 tests), including new
  `services::channel`, `services::direct_message`, `channel.rs`, and
  `direct_message.rs` integration tests for member-created channels, self-join,
  creator-managed channels, unarchive, DM conversation reuse, and DM
  hide/reappear-on-new-message.
- `cd desktop && pnpm typecheck && pnpm test` passes (25 tests).
- `cd web && pnpm typecheck && pnpm build` succeeds.
- Manual check (Playwright against a local server + `web` dev server): login
  with one org auto-redirects to `#general`; created a private channel with
  `+`, verified topic/purpose edit, archive, and unarchive via the channel
  settings modal; started a DM with a second test user, confirmed the sidebar
  and message-pane title resolve display names (not raw IDs), and confirmed
  hiding a DM removes it from the sidebar.

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

- `cargo fmt --all` passes.
- `cargo check --workspace` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo nextest run --workspace` passes (259 tests), including new
  `server_admin::server_admin_can_delete_user`,
  `server_admin::server_admin_cannot_delete_last_admin`,
  `server_admin::server_admin_cannot_delete_user_with_messages`, and
  `admin::team_members_and_rooms_can_be_managed`.
- `cd desktop && pnpm typecheck && pnpm test` passes (25 tests).
- `cd web && pnpm typecheck && pnpm build` succeeds.
- Manual check (Playwright against a local server + `web` dev server, isolated
  throwaway database): registered a first user (auto-promoted server admin);
  created a second user via the `EditUserModal` create flow and saw the
  generated password notice; opened the edit modal and exercised promote,
  password reset, and the danger-zone delete confirmation; invited the second
  user into the organization via `OrgAdminMembers.tsx` and changed their role;
  created a team in `OrgAdminTeams.tsx`, expanded its management panel, and
  added/removed a member and a room; confirmed the `ServerAdminShell` and
  `OrgAdminShell` "Back" links navigate to the chat UI; confirmed the mobile
  hamburger toggle shows/hides the `OrgAdminShell` sidebar at a 375px
  viewport.
- This manual pass caught and fixed a pre-existing bug: `OrgAdminShell.tsx`'s
  tab `NavLink`s used relative paths (e.g. `to="settings"`), so clicking a tab
  from a nested admin route appended to the current path instead of replacing
  it (e.g. `/admin/members` → `/admin/members/settings`), eventually
  navigating to a URL with no matching route. Tabs now build an absolute path
  from `organizationId` (`/org/{id}/admin/{tab.path}`).

### Verification

- Backend checks and integration tests pass.
- Frontend type checks and tests pass.
- Manual check: server admin users can be created and edited in a modal; org
  admin can manage members/roles/permissions/emoji/teams; admin back links
  return to chat.

---

## Phase 5 — Shell and Navigation

### Issues

- [ISSUES12](ISSUES12.md) — Collapsible sidebar (full ↔ narrow).
- [ISSUES13](ISSUES13.md) — Sidebar on mobile.
- [ISSUES14](ISSUES14.md) — Admin menu icons in top bar.

### Goals

1. Make the main chat sidebar collapsible to a narrow dock on desktop.
2. Expose a mobile entry point so users can open the sidebar on small screens.
3. Move admin navigation to icon buttons in the top bar with tooltips.

### Order of work

1. **Mobile sidebar (ISSUES13)**
   - Add a hamburger toggle to `Shell.tsx` that passes `mobileOpen`/`onClose` to `Sidebar`.
   - Ensure the mobile drawer overlays the content and closes on selection.
2. **Collapsible sidebar (ISSUES12)**
   - Add a collapse/expand toggle to `Sidebar` and a narrow variant (`w-16`).
   - Replace channel/DM text labels with initials/icons when collapsed.
   - Persist the collapsed state in `localStorage` (or server profile after ISSUES17).
3. **Admin top-bar icons (ISSUES14)**
   - Choose Font Awesome free icons for each admin tab.
   - Render icon buttons in the top-left of `ServerAdminShell.tsx` and `OrgAdminShell.tsx`.
   - Keep text labels as tooltips and keep active-state styling.

### Cross-phase impact

- Collapse state persistence may be promoted to server profile in ISSUES17.
- Admin icon changes affect the same shells used in Phase 4.

### Verification

- `cd desktop && pnpm typecheck && pnpm test` passes.
- `cd web && pnpm typecheck && pnpm build` succeeds.
- Manual check: collapse/expand sidebar on desktop; open sidebar on a 375px viewport; hover admin icons to see tooltips.

---

## Phase 6 — Home and User Profile

### Issues

- [ISSUES15](ISSUES15.md) — Single-organization home redirect.
- [ISSUES16](ISSUES16.md) — Organization home unread-messages view.
- [ISSUES17](ISSUES17.md) — Server-stored theme preference.

### Goals

1. Redirect single-organization users to a meaningful home view.
2. Build an organization home page that surfaces unread channels and DMs.
3. Persist the user's theme preference in their server profile.

### Order of work

1. **Single-org redirect (ISSUES15)**
   - Reproduce the reported blank `/org` view.
   - Update `AuthScreen.tsx` or the router index route to send single-org users to the last-selected channel or `#general`.
2. **Unread home view (ISSUES16)**
   - Create `OrgIndex.tsx` rendered by `/org`.
   - List channels and DMs with unread counts and links to each conversation.
   - Use existing `useReadState` and conversation-list hooks.
3. **Server-stored theme (ISSUES17)**
   - Add `theme` to the `users` table, domain `User`, and profile update API.
   - Load the server theme after login and save changes from `Settings.tsx`.

### Cross-phase impact

- ISSUES16 consumes the unread state introduced in ADR-015.
- ISSUES17 extends user profile data; may also store collapse preference from ISSUES12.

### Verification

- Backend integration tests for theme/profile update.
- Frontend type checks and tests pass.
- Manual check: single-org user lands in `#general`; `/org` shows unread list; theme survives logout/login.

---

## Phase 7 — Messages and Composer

### Issues

- [ISSUES10](ISSUES10.md) — Submitted message duplicated.
- [ISSUES18](ISSUES18.md) — Add message delete option.
- [ISSUES19](ISSUES19.md) — Remove Markdown preview from composer.

### Goals

1. Fix message duplication on send.
2. Add author/admin message deletion to the UI.
3. Remove the obsolete Markdown preview toggle from the Tiptap composer.

### Order of work

1. **Reproduce and fix duplication (ISSUES10)**
   - Add a test that sends messages rapidly or simulates WebSocket delivery before REST response.
   - Implement client-side deduplication for the optimistic-vs-real-message race.
2. **Message delete (ISSUES18)**
   - Add `deleteMessage` to `desktop/src/api/messages.ts` and `useMessages.ts`.
   - Add a Delete action to `MessageItem.tsx` with confirmation.
   - Confirm the backend soft-delete and any missing `message.deleted` broadcast.
3. **Remove preview (ISSUES19)**
   - Remove `showPreview` state and UI from `Composer.tsx`.
   - Update or remove preview tests.

### Cross-phase impact

- ISSUES18 may need a new WebSocket event if one does not exist.
- ISSUES19 simplifies the composer before further toolbar changes.

### Verification

- Unit/integration test for duplication scenario passes.
- Manual check: send a message rapidly and see one copy; delete a message and see `[deleted]`; no preview toggle remains.

---

## Phase 8 — Direct Messages and Server Admin Completion

### Issues

- [ISSUES20](ISSUES20.md) — Finish direct messages modal.
- [ISSUES21](ISSUES21.md) — Finish CRUD for server admins view.

### Goals

1. Make the "New message" DM modal actually list users.
2. Complete server-admin CRUD (promote + demote) in the admin UI.

### Order of work

1. **DM modal (ISSUES20)**
   - Diagnose why `StartDmModal.tsx` opens with no users listed.
   - Ensure `useOrgMemberContext` is available and populated in the sidebar/chat shell.
   - Add a search/filter input and verify multi-select DM creation.
2. **Server admins CRUD (ISSUES21)**
   - Add a demote action to each admin row in `ServerAdminAdmins.tsx`.
   - Add confirmation dialogs and handle the last-admin server error.
   - Add search/filter to the admin list.

### Cross-phase impact

- ISSUES20 relies on org member loading, also used by mention autocomplete.
- ISSUES21 overlaps with the user editor from Phase 4; consider unifying admin management UX.

### Verification

- Manual check: start a DM with another user from the sidebar `+` button.
- Manual check: promote a user to server admin and then demote them.

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
