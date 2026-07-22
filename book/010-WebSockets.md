# 010 - WebSockets

## Role

WebSockets deliver real-time events to connected clients. They are used for message delivery, presence, typing indicators, and other live updates. State-changing operations still go through the REST API; WebSockets carry events and lightweight signals only.

## Connection

- Endpoint: `wss://<host>/api/v1/ws`.
- The connection is authenticated by the same `ruckchat_session` cookie as REST requests.
- The server rejects unauthenticated connection attempts with `1008 Policy Violation`.
- Clients should reconnect with exponential backoff after a disconnect.

## Protocol

Messages are JSON objects with the following envelope:

```json
{
  "type": "message.created",
  "id": "event-id-uuid",
  "timestamp": "2026-07-21T14:30:00Z",
  "payload": { ... }
}
```

## Client-to-Server Events

| Type | Payload | Description |
|------|---------|-------------|
| `ping` | `{}` | Keep-alive; server responds with `pong`. |
| `typing.started` | `{ conversation_id, parent_id? }` | User started typing. |
| `typing.stopped` | `{ conversation_id }` | User stopped typing. |
| `presence.set` | `{ status: "online" \| "away" \| "dnd" }` | Update own presence. |

## Server-to-Client Events

| Type | Payload | Description |
|------|---------|-------------|
| `pong` | `{ server_time }` | Response to ping. |
| `message.created` | Message object | A new message was posted. |
| `message.updated` | Message object | A message was edited. |
| `message.deleted` | `{ id, conversation_id }` | A message was deleted. |
| `reaction.updated` | `{ message_id, emoji, count, user_reacted }` | Reaction added or removed. |
| `typing.updated` | `{ conversation_id, user_id, parent_id? }` | Typing state changed. |
| `presence.updated` | `{ user_id, status }` | Presence state changed. |
| `channel.updated` | Channel object | Channel metadata changed. |
| `channel.member_joined` | `{ channel_id, user_id }` | User joined a channel. |
| `channel.member_left` | `{ channel_id, user_id }` | User left a channel. |
| `server.restart` | `{}` | Server is restarting; client should reconnect. |

## Event Routing

- The WebSocket manager maintains a map of user IDs to active connections.
- When an event is published, the manager determines the target user set based on the conversation and organization.
- A user with multiple connected devices receives the event on each connection.
- Events are not persisted; clients fetch history via REST after reconnecting.

## Presence

- Presence is tracked in-memory for active WebSocket connections.
- Status values: `online`, `away`, `dnd`, `offline`.
- A user is `offline` when they have no active connections.
- Presence changes are broadcast to organization members.

## Typing Indicators

- `typing.started` is emitted when the composer gains input.
- `typing.stopped` is emitted when the composer is cleared or after a timeout (default 5 seconds).
- The server debounces and deduplicates typing events before broadcasting.

## Scaling Limits

- v1 assumes a single server process. All WebSocket connections terminate on that process.
- The in-memory connection map is sufficient for small-to-medium deployments.
- Horizontal scaling is a post-MVP concern and would require a shared pub/sub layer, which conflicts with the v1 anti-infrastructure stance.

## Backpressure

- If a client cannot keep up, the server drops non-critical events for that connection.
- Critical events (message creation, deletion) are queued briefly before disconnecting the slow client.
