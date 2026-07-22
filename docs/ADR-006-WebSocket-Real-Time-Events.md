# ADR 006: WebSocket Real-Time Events

## Status

Accepted

## Context

RuckChat needs to push live updates to clients when messages, reactions, typing
indicators, and presence change. REST polling would add latency and load, so a
persistent WebSocket connection is the natural fit for the Phase 5 server.

Key constraints:

- Services (message, reaction, user) should not depend on WebSocket plumbing.
- Multiple browser/desktop tabs can be open for the same user; all relevant
  tabs must receive events.
- Events must be scoped by conversation privacy: public channel messages go to
  all organization members, private channel/DM events go only to conversation
  members.
- The implementation must fit within a single server process for this phase; an
  external broker is deferred.

## Decision

Introduce a WebSocket server with the following architecture:

1. **Authenticated upgrade endpoint** at `GET /websocket`. The same session
   cookie and bearer-token extractor used by REST authenticates the connection.

2. **In-memory connection registry** (`ConnectionManager`) tracks sockets by
   connection id, by user, and by subscribed organization. It provides broadcast
   primitives: `broadcast_to_organization` and `broadcast_to_users`.

3. **Auto-subscribe on connect**: the socket handler loads the user's
   organization memberships and subscribes the connection to every organization.

4. **Event bus trait** (`EventBus`) decouples services from transport. The
   message and reaction services call `publish_message_created`,
   `publish_reaction_added`, etc. The WebSocket layer implements the trait as
   `WebSocketEventBus`, resolves recipients using repositories, and dispatches
   through `ConnectionManager`.

5. **Routing rules**:
   - Public channel message/reaction events are broadcast to the whole
     organization.
   - Private channel message/reaction events are broadcast to channel members.
   - DM message/reaction events are broadcast to conversation members.
   - Typing indicators are broadcast to conversation members only.
   - Presence changes are broadcast to every organization the user belongs to.

6. **JSON envelope protocol**: server-to-client events use a uniform envelope
   `{type, id, timestamp, payload}`. Client-to-server messages use a tagged enum
   (`typing`, `subscribe_organization`, `unsubscribe_organization`, `ping`).

7. **Testing strategy**: unit tests cover `ConnectionManager` routing; service
   unit tests use `MockEventBus`; integration tests use `tokio-tungstenite`
   against an in-process Axum server to verify end-to-end event delivery.

## Consequences

- Services stay transport-agnostic and remain unit-testable with a mock event
  bus.
- All real-time state lives in the server process, so scaling beyond a single
  instance will require replacing `ConnectionManager` with an external broker
  (Redis, NATS, etc.).
- Clients receive presence updates for their own connections; this is harmless
  and can be filtered client-side.
- Auto-subscription simplifies clients but means every connection receives all
  organization-wide public channel traffic by default.
