# 001 - Product

## Product Overview

RuckChat is a team communication application with a Rust server, a Tauri desktop client, and a Flutter mobile client. It provides real-time messaging, file sharing, search, notifications, and an extensible plugin system.

## MVP Features

The minimum viable product delivers the following capabilities:

1. **Authentication**
   - Email and password registration and login.
   - Session-based authentication with secure HTTP-only cookies.
   - Password reset via email token.

2. **Organizations**
   - Multi-tenant workspaces isolated by organization.
   - Organization creation, invitation flow, and member management.
   - Role-based permissions: owner, admin, member.

3. **Channels**
   - Public channels visible to all organization members.
   - Private channels with explicit membership.
   - Channel creation, archiving, and deletion.
   - Channel topics and purpose fields.

4. **Direct Messages**
   - One-to-one direct message conversations.
   - Group direct messages (up to a configurable limit, default 8).
   - DM creation by selecting users.

5. **Realtime Messaging**
   - Text messages with markdown formatting.
   - Message editing and deletion within a time window.
   - Message threads (replies to a parent message).
   - Reactions (emoji on messages).
   - Typing indicators and presence.
   - Message delivery through WebSockets.

6. **File Uploads**
   - Drag-and-drop and picker-based file uploads.
   - Storage on local filesystem (default) or S3-compatible object store.
   - File size and type limits configurable per organization.
   - Thumbnails for images.

7. **Search**
   - Full-text search over message content.
   - PostgreSQL full-text search is the v1 implementation.
   - Search scoping by channel, direct message, or organization.

8. **Notifications**
   - In-app unread counters and mention badges.
   - Email notifications for mentions and DMs when offline.
   - Desktop push notifications via the Tauri client.
   - Mobile push notifications via Flutter (planned for post-MVP if platform-specific services are required).

9. **Plugin SDK**
   - Native Rust plugin SDK for extending server behavior.
   - Plugin lifecycle: load, initialize, run, shutdown.
   - Hooks for message processing, commands, and notifications.

10. **Packaging**
    - Server released as a single executable and a Docker image.
    - Desktop client released for Linux, macOS, and Windows.
    - Mobile client released for Android and iOS.

## User Roles

| Role | Description |
|------|-------------|
| Owner | Full control over the organization, including billing and deletion. |
| Admin | Manage members, channels, and settings; cannot delete the organization. |
| Member | Participate in channels and DMs; create channels if permitted. |
| Guest | Read-only or limited access to specific channels (post-MVP). |

## Non-Goals for v1

- Federation with other chat servers.
- Video or audio calls.
- Built-in machine translation.
- Marketplace for third-party apps.
- Advanced analytics dashboards.

## Quality Expectations

- All MVP features have unit and integration tests.
- REST API changes are reflected in the OpenAPI specification.
- Every feature ships with updated documentation.
- The codebase passes `cargo fmt`, `cargo clippy`, and CI before merge.
