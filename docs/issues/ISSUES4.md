# ISSUES4 — Channel Creation and Management UI

## Source

> No method for creating channels (CRUD — Also public and private channels with user invite CRUD for private channels) — Modal like <https://github.com/block/buzz/blob/main/docs/assets/screenshots/create-channel.png> — open

## Research Summary

### Current state

- The backend supports creating channels via `POST /api/v1/organizations/{id}/channels` (`server/openapi.yaml:3152-3155`).
- `CreateChannelRequest` accepts `name` and `is_private`.
- `desktop/src/api/channels.ts` likely wraps this endpoint (not read directly, but `CreateChannelRequest` is exported from `types.ts`).
- The `Channel` domain model includes `topic`, `purpose`, `is_private`, and `archived_at` (`crates/ruckchat-domain/src/channel.rs:13-34`).
- `useChannels` loads the channel list for the active organization.
- The sidebar renders channels, but there is no visible "Add channel" button or modal.

### Gaps

1. **Create channel modal** — add a modal reachable from the sidebar (e.g., "+" next to "Channels") with fields for name, public/private, topic, and purpose.
2. **Private channel invites** — for private channels, provide a UI to invite members and manage membership after creation.
3. **Channel update/delete/archive** — add UI actions to edit topic/purpose, archive, or delete channels.
4. **Channel list UX** — distinguish public vs. private channels, archived channels, and unread state in the sidebar.
5. **Permissions** — ensure only users allowed by `ChannelService`/`AuthorizationService` see the create/edit actions.

### Affected files

- `desktop/src/components/Sidebar.tsx` — add channel section header and create button.
- `desktop/src/components/CreateChannelModal.tsx` (new) — create-channel form.
- `desktop/src/api/channels.ts` — verify existing create/update/archive endpoints are wired.
- `server/src/services/channel.rs` — may need invite/membership management endpoints if missing.
- `server/openapi.yaml` — add private-channel invite endpoints if not present.

## Open Questions

1. **Who can create channels?**
   - Any organization member (current backend test suggests members are forbidden; verify actual policy).
   - Only owners and admins.
   - Configurable per organization.

2. **How should private-channel membership be managed?**
   - Add/remove members via a dedicated modal after channel creation.
   - Invite users at creation time only.
   - Both.

3. **What happens to an archived channel?**
   - It stays in the sidebar in a collapsed "Archived" section.
   - It is hidden unless the user explicitly shows archived channels.

4. **Should channel deletion be supported, or only archiving?**
   - Archiving only (Slack-style).
   - Both archive and permanent delete for admins.

## Decisions

- Channel creation permission: any organization member can create channels.
- Private-channel membership: support both creation-time invites and post-creation member management.
- Archived channels: collapsed "Archived" section in the sidebar.
- Channel removal: archiving only; no permanent delete in the UI.
