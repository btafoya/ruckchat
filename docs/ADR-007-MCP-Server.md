# ADR 007: MCP Server

## Status

Accepted

## Context

RuckChat needs to expose a Model Context Protocol (MCP) server so that AI
assistants can read context and perform actions on behalf of authorized users.
The MCP server must reuse the same Rust process and PostgreSQL database as the
REST API and WebSocket server; it is not a separate service.

Key constraints:

- MCP authentication must use the same session mechanism as the REST API and
  WebSocket (session cookie or bearer token).
- MCP tools and resources must not bypass existing authorization rules.
- The implementation must stay within a single server process and avoid
  additional infrastructure.
- The server should use a standards-compliant MCP transport so clients can
  connect without custom protocol work.
- Sensitive actions such as posting messages should require explicit user
  confirmation by default.

## Decision

Introduce an MCP server with the following architecture:

1. **Transport**: Server-Sent Events (SSE) using the Streamable HTTP transport
   from the official `rmcp` Rust SDK (version 2.2.0), with the `server`,
   `server-side-http`, and `transport-streamable-http-server` features. The
   single endpoint `/mcp/v1/sse` accepts `POST` requests carrying JSON-RPC client
   messages and returns responses as SSE events. In stateful mode `rmcp` also
   supports `GET` requests on the same path to open a long-lived SSE stream for
   server-initiated messages, using the `Mcp-Session-Id` header to identify the
   session.

2. **Authentication**: The Axum route for `/mcp/v1/sse` runs the existing
   `AuthUser` extractor before handing the request to `rmcp`. Unauthenticated
   upgrades receive the standard 401 JSON error body and never create an MCP
   session.

3. **Authorization**: A new `McpService` in `server/src/services/mcp.rs`
   wraps the existing service layer. It never accesses repositories directly.
   Each tool call carries the authenticated `UserId` and delegates to
   `ChannelService`, `DirectMessageService`, `MessageService`, `UserService`,
   `OrganizationService`, and `AuthorizationService`, inheriting the same
   `Error::Forbidden` semantics as REST.

4. **Tools**: The v1 MCP server exposes six tools:
   - `list_channels`
   - `list_direct_messages`
   - `get_messages`
   - `search_messages`
   - `post_message`
   - `get_user_profile`

5. **Resources**: The v1 MCP server exposes four read-only resources:
   - `ruckchat://organization/{id}`
   - `ruckchat://channel/{id}`
   - `ruckchat://conversation/{id}`
   - `ruckchat://message/{id}`

6. **Search**: Full-text search uses the existing PostgreSQL `content_tsv`
   column on the `messages` table. A new `MessageRepository::search` method
   scopes results by organizations the caller belongs to and by conversation
   visibility (public channels, private channel membership, DM membership).

7. **Confirmation flow**: `post_message` checks the
   `MCP_REQUIRE_CONFIRMATION` setting (default `true`). When enabled and the
   request does not include `confirmed: true`, the tool returns a
   confirmation-pending response instead of posting. This is a simple,
   stateless guard for v1.

8. **Configuration**: Two settings control the MCP server:
   - `MCP_ENABLED` (default `true`) — whether the `/mcp/v1/sse` endpoint is
     mounted.
   - `MCP_REQUIRE_CONFIRMATION` (default `true`) — whether `post_message`
     requires an explicit confirmation flag.

9. **Module layout**: A new `server/src/mcp/` module contains:
   - `mod.rs` — module exports.
   - `server.rs` — `RuckChatMcpServer` implementing `rmcp`'s
     `ServerHandler`.
   - `tools.rs` — tool definitions and handlers.
   - `resources.rs` — resource definitions and read helpers.
   - `handler.rs` — Axum route handler that builds the Streamable HTTP service
     and wires the route into the Axum router. The authenticated `UserId` is
     injected into the request extensions; `RuckChatMcpServer` reads it from
     the `http::request::Parts` extensions passed through the JSON-RPC request
     context.

10. **Testing strategy**: Unit tests cover `McpService` authorization and the
    `post_message` confirmation flow using in-memory mock repositories.
    Integration tests open an SSE session to `/mcp/v1/sse` and verify tool
    calls end-to-end against an in-process Axum router.

## Consequences

- AI assistants interact with RuckChat through a standard protocol without
  custom API endpoints.
- The MCP server inherits all existing authorization and visibility rules,
  so it cannot read private channels or DMs the user is not a member of.
- All MCP state lives in the same process and database as the REST API; no
  additional infrastructure is required.
- The `rmcp` SDK simplifies SSE and JSON-RPC handling, but its API dictates the
  handler trait shape and feature flags. If the SDK becomes incompatible, the
  MCP module can be replaced with a manual SSE/JSON-RPC implementation without
  touching the service layer.
- `post_message` confirmation is coarse (a single flag) in v1. Future versions
  may adopt MCP sampling or a richer consent flow.
- MCP tools are read-only or message-posting only; destructive actions such as
  deleting messages or files are excluded by design.
