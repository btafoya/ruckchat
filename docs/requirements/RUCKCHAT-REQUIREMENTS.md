# RuckChat v1 Requirements

## 1. Goals and Scope

RuckChat is a self-hostable team chat platform. The v1 product is a single-server application with desktop and mobile clients. This document captures the functional and non-functional requirements derived from the RuckChat handbook.

### 1.1 In Scope

- User authentication and session management.
- Multi-tenant organizations with role-based access control.
- Public and private channels.
- Direct messages, including small groups.
- Real-time text messaging with threads, reactions, and editing.
- File uploads with local or S3-compatible storage.
- Full-text search over message content.
- In-app, email, and desktop notifications.
- Native Rust plugin SDK for server extensions.
- MCP server for AI assistant integration.
- Deployment as a single binary or Docker container.

### 1.2 Out of Scope

- Federation across servers.
- Audio or video calls.
- Built-in translation.
- Third-party app marketplace.
- Advanced analytics dashboards.
- Background mobile push notifications for v1.

## 2. Functional Requirements

### 2.1 Authentication

| ID | Requirement | Priority |
|----|-------------|----------|
| AUTH-001 | Users must register with a unique email address and a password. | Must |
| AUTH-002 | Passwords must be at least 10 characters by default. | Must |
| AUTH-003 | Passwords must be hashed with Argon2id before storage. | Must |
| AUTH-004 | Users must log in with email and password to receive a session cookie. | Must |
| AUTH-005 | Session cookies must be `HttpOnly`, `Secure` in production, and `SameSite=Lax`. | Must |
| AUTH-006 | Sessions must expire after a configurable idle period, default 30 days. | Must |
| AUTH-007 | Sessions must be invalidated on password change and explicit logout. | Must |
| AUTH-008 | Users must be able to request a password reset via email token. | Must |
| AUTH-009 | The system must provide a `GET /api/v1/auth/me` endpoint returning the current user. | Must |
| AUTH-010 | Rate limiting must be enforced per IP on authentication endpoints. | Must |

### 2.2 Users

| ID | Requirement | Priority |
|----|-------------|----------|
| USER-001 | Users must have a display name between 1 and 100 characters. | Must |
| USER-002 | Users must have a globally unique email address. | Must |
| USER-003 | Users must be able to update their display name and avatar. | Must |
| USER-004 | Users must be able to belong to multiple organizations. | Must |
| USER-005 | User avatars must be stored via the same file storage backend as attachments. | Should |

### 2.3 Organizations

| ID | Requirement | Priority |
|----|-------------|----------|
| ORG-001 | A user must be able to create an organization with a unique URL slug. | Must |
| ORG-002 | Organization slugs must be 3-63 characters and URL-safe. | Must |
| ORG-003 | The organization creator must be assigned the owner role. | Must |
| ORG-004 | Organization owners must be able to invite users by email. | Must |
| ORG-005 | Invited users must accept the invitation to become members. | Must |
| ORG-006 | Organization owners and admins must be able to remove members. | Must |
| ORG-007 | Organization owners must be able to delete the organization. | Must |
| ORG-008 | An organization must always retain at least one owner. | Must |
| ORG-009 | Organization members must have one of three roles: owner, admin, or member. | Must |
| ORG-010 | Admins must be able to manage channels and members but not delete the organization. | Must |

### 2.4 Channels

| ID | Requirement | Priority |
|----|-------------|----------|
| CH-001 | Organization members must be able to create public channels. | Must |
| CH-002 | Organization members must be able to create private channels. | Must |
| CH-003 | Channel names must be unique within an organization and limited to lowercase letters, numbers, and hyphens. | Must |
| CH-004 | Public channels must be visible and readable by all organization members. | Must |
| CH-005 | Private channels must be readable only by explicit members. | Must |
| CH-006 | Users must join a public channel before posting in it. | Must |
| CH-007 | Users must be invited to a private channel before viewing or posting. | Must |
| CH-008 | Channel creators and admins must be able to archive or delete a channel. | Must |
| CH-009 | Channels must support a topic and purpose field. | Should |
| CH-010 | Channel membership changes must be broadcast to relevant clients in real time. | Must |

### 2.5 Direct Messages

| ID | Requirement | Priority |
|----|-------------|----------|
| DM-001 | A user must be able to start a one-to-one direct message with another organization member. | Must |
| DM-002 | A user must be able to start a group direct message with up to 7 other members (total 8). | Must |
| DM-003 | DM membership must be fixed at creation time in v1. | Must |
| DM-004 | A DM must not be created if an identical DM already exists. | Should |
| DM-005 | DM membership must be restricted to organization members. | Must |

### 2.6 Messaging

| ID | Requirement | Priority |
|----|-------------|----------|
| MSG-001 | Users must be able to send text messages in channels and DMs. | Must |
| MSG-002 | Message content must support Markdown formatting. | Must |
| MSG-003 | Message content must be between 1 and 4000 characters by default. | Must |
| MSG-004 | Users must be able to edit their own messages within a configurable time window. | Must |
| MSG-005 | Users must be able to delete their own messages. | Must |
| MSG-006 | Admins and owners must be able to delete any message. | Must |
| MSG-007 | Deleted messages must retain their ID and metadata but replace content with a placeholder. | Must |
| MSG-008 | Users must be able to reply to a message in a thread. | Must |
| MSG-009 | Thread replies must be visible in the thread view and optionally in the channel. | Should |
| MSG-010 | Users must be able to add and remove emoji reactions on messages. | Must |
| MSG-011 | A user must only add one instance of a given emoji reaction to a message. | Must |
| MSG-012 | Messages must deliver to connected clients in real time via WebSocket. | Must |
| MSG-013 | Users must be able to mention other users with `@username`. | Must |
| MSG-014 | Mentioned users must receive a notification. | Must |

### 2.7 File Uploads

| ID | Requirement | Priority |
|----|-------------|----------|
| FILE-001 | Users must be able to upload files through the REST API. | Must |
| FILE-002 | Files must be attributed to an organization and uploading user. | Must |
| FILE-003 | File metadata must include name, MIME type, size, and storage path. | Must |
| FILE-004 | Files must be stored on the local filesystem by default. | Must |
| FILE-005 | The system must optionally support S3-compatible object storage. | Should |
| FILE-006 | Per-file size limits must be configurable, default 25 MB. | Must |
| FILE-007 | Organization storage quotas must be configurable, default 10 GB. | Should |
| FILE-008 | Images must have thumbnail generation. | Should |
| FILE-009 | Files must be served through authenticated endpoints, not direct static paths. | Must |
| FILE-010 | File uploads must validate MIME type and strip executable permissions. | Must |

### 2.8 Search

| ID | Requirement | Priority |
|----|-------------|----------|
| SEARCH-001 | Users must be able to search message content by keyword. | Must |
| SEARCH-002 | Search must use PostgreSQL full-text search by default. | Must |
| SEARCH-003 | Search results must be scoped to organizations the user belongs to. | Must |
| SEARCH-004 | Users must be able to scope search to a specific channel or DM. | Should |
| SEARCH-005 | Search must return paginated results. | Must |
| SEARCH-006 | Search must respect channel and DM membership permissions. | Must |

### 2.9 Notifications

| ID | Requirement | Priority |
|----|-------------|----------|
| NOTIF-001 | Users must see unread badges on channels and DMs. | Must |
| NOTIF-002 | Users must receive in-app notifications for mentions and thread replies. | Must |
| NOTIF-003 | The system must send email notifications for mentions and DMs when the user is offline. | Should |
| NOTIF-004 | Email notifications must be sent only after the user has been offline for more than 5 minutes. | Should |
| NOTIF-005 | Desktop clients must show native OS notifications for mentions and DMs. | Must |
| NOTIF-006 | Users must be able to disable email and desktop notifications. | Should |
| NOTIF-007 | Notification events must be routed through WebSocket for real-time updates. | Must |

### 2.10 WebSockets

| ID | Requirement | Priority |
|----|-------------|----------|
| WS-001 | The server must expose a single WebSocket endpoint authenticated by session cookie. | Must |
| WS-002 | WebSocket messages must use a JSON envelope with type, id, timestamp, and payload. | Must |
| WS-003 | The server must broadcast message creation, update, and deletion events. | Must |
| WS-004 | The server must broadcast reaction changes. | Must |
| WS-005 | The server must broadcast typing indicators. | Must |
| WS-006 | The server must broadcast presence changes. | Must |
| WS-007 | The server must track active connections per user in memory. | Must |
| WS-008 | Clients must reconnect automatically with exponential backoff. | Should |
| WS-009 | Server restart must close connections with a `server.restart` event. | Should |
| WS-010 | The server must support a ping/pong heartbeat. | Must |

### 2.11 REST API

| ID | Requirement | Priority |
|----|-------------|----------|
| API-001 | The API must be served under `/api/v1`. | Must |
| API-002 | All requests and responses must use JSON. | Must |
| API-003 | The API must use UUIDs in resource paths. | Must |
| API-004 | Date-time values must be ISO 8601 strings. | Must |
| API-005 | The API must return structured error bodies with code, message, and optional details. | Must |
| API-006 | List endpoints must support pagination via cursor or offset. | Must |
| API-007 | The API must be documented in an OpenAPI specification. | Must |
| API-008 | Every endpoint change must update the OpenAPI specification. | Must |

### 2.12 MCP Server

| ID | Requirement | Priority |
|----|-------------|----------|
| MCP-001 | The server must expose an MCP endpoint over Server-Sent Events. | Should |
| MCP-002 | MCP authentication must use the same session cookie as the REST API. | Must |
| MCP-003 | MCP tools must only expose data and actions the authenticated user can access. | Must |
| MCP-004 | MCP must provide tools to list channels, DMs, messages, and users. | Should |
| MCP-005 | MCP `post_message` tool must require user confirmation by default. | Should |

### 2.13 Plugin SDK

| ID | Requirement | Priority |
|----|-------------|----------|
| PLUGIN-001 | The server must load native Rust dynamic libraries from a configured plugin directory. | Should |
| PLUGIN-002 | Plugins must receive lifecycle hooks: load, initialize, run, shutdown. | Should |
| PLUGIN-003 | Plugins must subscribe to message and notification events. | Should |
| PLUGIN-004 | Plugins must register slash commands. | Should |
| PLUGIN-005 | Plugins must interact with the server only through the host API. | Must |
| PLUGIN-006 | Plugin failures must not crash the server. | Should |

### 2.14 Clients

#### Desktop

| ID | Requirement | Priority |
|----|-------------|----------|
| DESK-001 | The desktop client must be a Tauri + React application. | Must |
| DESK-002 | The desktop client must support Linux, macOS, and Windows. | Must |
| DESK-003 | The desktop client must use native OS notifications. | Must |
| DESK-004 | The desktop client must maintain a WebSocket connection while running. | Must |
| DESK-005 | The desktop client must preserve draft messages across reconnects. | Should |

#### Mobile

| ID | Requirement | Priority |
|----|-------------|----------|
| MOB-001 | The mobile client must be a Flutter application. | Must |
| MOB-002 | The mobile client must target Android and iOS. | Must |
| MOB-003 | The mobile client must support foreground notifications. | Must |
| MOB-004 | The mobile client must reconnect and fetch missed history on resume. | Must |
| MOB-005 | Background push notifications are post-MVP. | Won't |

## 3. Non-Functional Requirements

### 3.1 Architecture

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-001 | The server must be a single Rust process. | Must |
| NFR-002 | The system must use PostgreSQL as the sole database. | Must |
| NFR-003 | The system must not require Redis, Kafka, Elasticsearch, Kubernetes, or microservices in v1. | Must |
| NFR-004 | The server must expose REST and WebSocket endpoints from the same process. | Must |
| NFR-005 | All shared state must live in PostgreSQL or in-memory within the server process. | Must |

### 3.2 Performance

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-006 | The server must handle at least 1,000 concurrent WebSocket connections on commodity hardware. | Should |
| NFR-007 | Message send latency from client to broadcast must be under 100 ms at p95. | Should |
| NFR-008 | API endpoints must respond within 200 ms at p95 for typical operations. | Should |
| NFR-009 | Full-text search must return results within 500 ms at p95 for organizations under 100,000 messages. | Should |

### 3.3 Scalability

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-010 | The v1 architecture must scale vertically. | Must |
| NFR-011 | Horizontal scaling is explicitly out of scope for v1. | Won't |
| NFR-012 | A single server instance must support at least one organization with a few thousand users. | Should |

### 3.4 Reliability

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-013 | The server must gracefully shut down on SIGTERM, completing open requests. | Should |
| NFR-014 | WebSocket connections must reconnect automatically on transient failures. | Should |
| NFR-015 | Database migrations must apply automatically on startup by default. | Must |
| NFR-016 | Failed database operations must not corrupt message history. | Must |

### 3.5 Security

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-017 | Passwords must never be stored or transmitted in plaintext. | Must |
| NFR-018 | Session tokens must be hashed before database storage. | Must |
| NFR-019 | All production traffic must use HTTPS and WSS. | Must |
| NFR-020 | SQL queries must be parameterized to prevent injection. | Must |
| NFR-021 | Uploaded files must be validated by MIME type and size before storage. | Must |
| NFR-022 | Uploaded files must not be served from the web root. | Must |
| NFR-023 | Error responses must not leak stack traces or database details. | Must |
| NFR-024 | Secrets must be loaded from environment variables, never committed. | Must |

### 3.6 Maintainability

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-025 | The codebase must pass `cargo fmt` and `cargo clippy` without warnings. | Must |
| NFR-026 | Every feature must include unit and integration tests. | Must |
| NFR-027 | Every API change must update the OpenAPI specification. | Must |
| NFR-028 | Every feature must update relevant documentation, including the handbook if affected. | Must |
| NFR-029 | The server code must separate handlers, services, and repositories. | Must |

### 3.7 Observability

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-030 | The server must expose `/health` and `/health/ready` endpoints. | Must |
| NFR-031 | The server must emit structured logs using `tracing`. | Must |
| NFR-032 | Prometheus metrics must be available at `/metrics` when enabled. | Should |
| NFR-033 | Logs must not contain passwords or raw session tokens. | Must |

### 3.8 Usability

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-034 | A new operator must be able to install and run RuckChat in under 10 minutes. | Should |
| NFR-035 | A developer must be able to build and run the stack locally in under 5 minutes. | Should |
| NFR-036 | End users must send and receive messages without training. | Must |
| NFR-037 | Desktop and mobile clients must behave consistently where applicable. | Should |

### 3.9 Accessibility

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-038 | The desktop client must support full keyboard navigation. | Should |
| NFR-039 | Interactive elements must have visible focus indicators. | Should |
| NFR-040 | Color contrast must meet WCAG 2.1 AA. | Should |

### 3.10 Compatibility

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-041 | The server must compile on stable Rust. | Must |
| NFR-042 | PostgreSQL 15 or later must be supported. | Must |
| NFR-043 | The Docker image must run on Linux amd64 and arm64. | Should |

## 4. User Stories and Acceptance Criteria

### 4.1 Registration and Login

**Story:** As a new user, I want to register with my email so that I can use RuckChat.

**Acceptance Criteria:**
- Given a valid email and password, when I register, then an account is created and I am logged in.
- Given an already-used email, when I register, then I receive a conflict error.
- Given a password shorter than 10 characters, when I register, then I receive a validation error.

### 4.2 Create a Channel

**Story:** As an organization member, I want to create a channel so that my team can discuss a topic.

**Acceptance Criteria:**
- Given a channel name that is unique in the organization, when I create it, then the channel appears in the channel list for all members.
- Given a duplicate channel name, when I create it, then I receive a conflict error.
- Given a private channel, when I create it, then only invited members see it.

### 4.3 Send a Real-Time Message

**Story:** As a user, I want to send a message in a channel so that others see it immediately.

**Acceptance Criteria:**
- Given I am in a channel, when I send a message, then it appears in my message list and the lists of all connected members.
- Given another member is offline, when I send a message, then they see it after reconnecting and fetching history.
- Given I mention another user, when the message is sent, then that user receives a notification.

### 4.4 Upload a File

**Story:** As a user, I want to upload a file to a conversation so that others can download it.

**Acceptance Criteria:**
- Given a file under the size limit, when I upload it, then it appears as an attachment in the conversation.
- Given an oversized file, when I upload it, then I receive a validation error.
- Given I am not authenticated, when I request the file download URL, then I am denied.

### 4.5 Search Messages

**Story:** As a user, I want to search message history so that I can find past discussions.

**Acceptance Criteria:**
- Given a keyword that exists in a message I can access, when I search, then the message appears in results.
- Given a keyword in a private channel I do not belong to, when I search, then that message does not appear.
- Given more results than the page limit, when I search, then I can paginate through results.

## 5. Open Questions

1. Should guest accounts be supported in v1? (Currently excluded.)
2. Should message editing be allowed indefinitely or within a time window? (Default: configurable, suggest 24 hours.)
3. Should deleted messages be visible as "deleted" to all members or removed entirely? (Default: retained as deleted placeholders.)
4. What languages should full-text search support beyond English? (Default: English only in v1.)
5. Should the mobile client require platform-specific push notification services before v1 ships? (Default: no.)
6. Should plugins be distributed as signed binaries? (Default: signing is post-MVP.)
7. Is a web client required, or are desktop and mobile sufficient? (Default: desktop and mobile only.)
