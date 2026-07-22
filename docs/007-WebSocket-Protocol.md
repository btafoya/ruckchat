# 007: WebSocket Protocol

## Purpose

Define the real-time event protocol used by `GET /websocket`. WebSocket
connections let clients receive live message, reaction, typing, and presence
updates without polling the REST API.

## Requirements

- Authenticate using the same session cookie or bearer token as REST.
- Deliver events in a uniform JSON envelope that clients can dispatch by type.
- Accept a small set of client-to-server actions (typing, subscribe,
  unsubscribe, heartbeat ping).
- Scope broadcasts by conversation privacy and membership.

## Design

### Connection

Connect to `ws://<base_url>/websocket` with either:

- `Cookie: ruckchat_session=<token>`
- `Authorization: Bearer <token>`

On success the server sends `connection.established` containing the
authenticated user id. The connection is automatically subscribed to every
organization the user belongs to.

### Server-to-client envelope

```json
{
  "type": "message.created",
  "id": "<event uuid>",
  "timestamp": "2026-07-22T12:34:56Z",
  "payload": { ... }
}
```

Event types and payloads:

| Type | Payload |
|------|---------|
| `connection.established` | `{ user_id }` |
| `message.created` | `{ message }` |
| `message.updated` | `{ message }` |
| `message.deleted` | `{ message }` |
| `reaction.added` | `{ reaction }` |
| `reaction.removed` | `{ message_id, user_id, emoji }` |
| `typing.updated` | `{ user_id, conversation_id, conversation_type }` |
| `presence.updated` | `{ user_id, status }` (`online` or `offline`) |
| `error` | `{ code, message }` |

### Client-to-server messages

Client messages are flat JSON objects tagged by `type`:

```json
{ "type": "typing", "conversation_id": "...", "conversation_type": "channel" }
{ "type": "subscribe_organization", "organization_id": "..." }
{ "type": "unsubscribe_organization", "organization_id": "..." }
{ "type": "ping" }
```

`conversation_type` is `channel` or `direct_message`.

### Broadcast rules

- Public channel message/reaction events go to all members of the channel's
  organization.
- Private channel message/reaction events go to channel members only.
- DM message/reaction events go to conversation members.
- Typing events go to conversation members only.
- Presence events go to every organization the affected user belongs to.

## Acceptance Criteria

- A client connecting with a valid token receives `connection.established`.
- Posting a message through REST causes connected clients to receive
  `message.created`.
- Adding and removing reactions causes `reaction.added` and `reaction.removed`.
- Sending a `typing` client message causes `typing.updated`.
- Multiple connections for the same user all receive targeted events.
