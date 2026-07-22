# 010 - WebSockets

## Role

WebSockets deliver real-time events to connected clients. REST remains the
source of truth for state-changing operations; the socket carries events and
lightweight signals only.

## Connection

- Endpoint: `ws://<host>:<port>/websocket` (use TLS in production).
- Authentication uses the same `ruckchat_session` cookie or
  `Authorization: Bearer <token>` header as REST.
- The server sends a `connection.established` event and then a
  `presence.updated` `online` announcement for the connecting user.
- Clients should reconnect with exponential backoff after a disconnect.

## Protocol

Server-to-client events use a uniform envelope:

```json
{
  "type": "message.created",
  "id": "<event uuid>",
  "timestamp": "2026-07-22T12:34:56Z",
  "payload": { ... }
}
```

Client-to-server messages are flat JSON objects:

```json
{ "type": "typing", "conversation_id": "...", "conversation_type": "channel" }
{ "type": "subscribe_organization", "organization_id": "..." }
{ "type": "unsubscribe_organization", "organization_id": "..." }
{ "type": "ping" }
```

`conversation_type` is `channel` or `direct_message`.

## Server-to-Client Events

| Type | Payload | Description |
|------|---------|-------------|
| `connection.established` | `{ user_id }` | Socket is ready. |
| `message.created` | `{ message }` | A new message was posted. |
| `message.updated` | `{ message }` | A message was edited. |
| `message.deleted` | `{ message }` | A message was soft-deleted. |
| `reaction.added` | `{ reaction }` | A reaction was added to a message. |
| `reaction.removed` | `{ message_id, user_id, emoji }` | A reaction was removed. |
| `typing.updated` | `{ user_id, conversation_id, conversation_type }` | A user is typing. |
| `presence.updated` | `{ user_id, status }` | `online` or `offline`. |
| `error` | `{ code, message }` | Invalid client message or transport error. |

## Event Routing

- Public channel message/reaction events are broadcast to every member of the
  channel's organization.
- Private channel message/reaction events are broadcast to channel members only.
- DM message/reaction events are broadcast to conversation members.
- Typing indicators are broadcast to conversation members only.
- Presence changes are broadcast to every organization the affected user belongs to.
- A user with multiple open connections receives targeted events on each one.

## Presence

- Presence is tracked in memory based on open WebSocket connections.
- `online` is emitted when a user's first connection opens.
- `offline` is emitted when the user's last connection closes.

## Scaling Limits

- v1 assumes a single server process. All WebSocket connections terminate on that
  process and all state lives in memory.
- Horizontal scaling is a post-MVP concern and would require a shared pub/sub
  layer, which conflicts with the v1 anti-infrastructure stance.
