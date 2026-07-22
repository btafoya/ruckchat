# 008-MCP-Server

## Purpose

RuckChat exposes a Model Context Protocol (MCP) server so that AI assistants
can read context from and perform a limited set of actions on behalf of an
authorized user. The MCP server reuses the same Rust server process and
PostgreSQL database as the REST API and WebSocket endpoints.

## Requirements

- MCP-001: The server must expose an MCP endpoint over Server-Sent Events.
- MCP-002: MCP authentication must use the same session cookie or bearer token
  as the REST API and WebSocket.
- MCP-003: MCP tools and resources must only expose data and actions the
  authenticated user is authorized to access.
- MCP-004: MCP must provide tools to list channels, DMs, messages, users, and
  to search message history.
- MCP-005: MCP `post_message` must require user confirmation by default.
- MCP-006: MCP resources must use the `ruckchat://` URI scheme.

## Design

### Transport

- v1 uses the Streamable HTTP transport from the `rmcp` Rust SDK.
- Endpoint: `/mcp/v1/sse`.
- `POST` requests carry JSON-RPC client messages and return responses as SSE
  events.
- `GET` requests open a long-lived SSE stream for server-initiated messages when
  the client sends the session ID in the `Mcp-Session-Id` header.

### Authentication

- The Axum route runs the existing `AuthUser` extractor.
- Unauthenticated requests receive a 401 JSON error before an MCP session is
  created.
- Authenticated sessions carry the caller's `UserId` into the MCP handler.

### Authorization

- The MCP layer does not access repositories directly.
- A new `McpService` delegates every tool call to the existing service layer
  (`ChannelService`, `DirectMessageService`, `MessageService`, `UserService`,
  `OrganizationService`, and `AuthorizationService`).
- Every tool call is subject to the same organization membership, channel
  membership, and DM membership checks as the REST API.

### Tools

| Tool | Input | Behavior |
|------|-------|----------|
| `list_channels` | `{ organization_id }` | List channels visible to the user in the organization. |
| `list_direct_messages` | `{ organization_id }` | List DM conversations for the user in the organization. |
| `get_messages` | `{ conversation_id, conversation_type, limit?, offset? }` | Fetch recent messages the user can read. |
| `search_messages` | `{ organization_id, query, limit?, offset? }` | Search message content with PostgreSQL full-text search. |
| `post_message` | `{ conversation_id, conversation_type, content, parent_id?, confirmed? }` | Post a message; requires `confirmed: true` when `MCP_REQUIRE_CONFIRMATION` is enabled. |
| `get_user_profile` | `{ user_id }` | Read a user profile if visible to the caller. |

### Resources

| Resource | URI | Behavior |
|----------|-----|----------|
| Organization | `ruckchat://organization/{id}` | Read organization metadata if the user is a member. |
| Channel | `ruckchat://channel/{id}` | Read channel metadata if visible to the user. |
| Conversation | `ruckchat://conversation/{id}` | Read channel or DM metadata if visible. |
| Message | `ruckchat://message/{id}` | Read a single message if visible. |

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `MCP_ENABLED` | `true` | Enable the MCP endpoint. |
| `MCP_REQUIRE_CONFIRMATION` | `true` | Require explicit confirmation before posting messages. |

## Acceptance Criteria

1. An authenticated client can open an SSE session at `/mcp/v1/sse`.
2. The `list_channels` tool returns only channels visible to the caller.
3. The `get_messages` tool returns messages only for conversations the caller
   can read.
4. The `search_messages` tool returns full-text results scoped to the caller's
   organizations and respecting private channel / DM visibility.
5. The `post_message` tool succeeds when confirmation is disabled or when
   `confirmed: true` is provided; it returns a confirmation-needed response
   when confirmation is enabled and `confirmed` is false.
6. The MCP endpoint is absent when `MCP_ENABLED` is false.
7. Unit tests cover `McpService` authorization and confirmation logic.
8. Integration tests cover SSE connection, tool calls, and confirmation flow.
9. OpenAPI and handbook documentation are updated.
