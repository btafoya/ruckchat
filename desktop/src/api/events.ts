import type { Message } from '../api/types';

export type PresenceStatus = 'online' | 'offline';

// These `type` discriminants match the server's `#[serde(tag = "type",
// rename_all = "snake_case")]` on `ServerEvent` (server/src/services/events.rs)
// verbatim - that's the tag on the *inner* payload object, which is what
// clients actually switch on, not the outer envelope's dotted `event_type()`
// string (e.g. "message.created"). The two must not be confused.

export interface MessageCreatedEvent {
  type: 'message_created';
  message: Message;
}

export interface MessageUpdatedEvent {
  type: 'message_updated';
  message: Message;
}

export interface MessageDeletedEvent {
  type: 'message_deleted';
  message: Message;
}

export interface ReactionAddedEvent {
  type: 'reaction_added';
  message_id: string;
  user_id: string;
  emoji: string;
}

export interface ReactionRemovedEvent {
  type: 'reaction_removed';
  message_id: string;
  user_id: string;
  emoji: string;
}

export interface TypingEvent {
  type: 'typing';
  user_id: string;
  conversation_id: string;
  conversation_type: 'channel' | 'direct_message';
}

export interface PresenceEvent {
  type: 'presence';
  user_id: string;
  status: PresenceStatus;
}

export interface ConnectionEstablishedEvent {
  type: 'connection_established';
  user_id: string;
}

export interface ReadStateUpdatedEvent {
  type: 'read_state_updated';
  conversation_id: string;
  message_ids: string[];
}

export type ServerEvent =
  | MessageCreatedEvent
  | MessageUpdatedEvent
  | MessageDeletedEvent
  | ReactionAddedEvent
  | ReactionRemovedEvent
  | TypingEvent
  | PresenceEvent
  | ConnectionEstablishedEvent
  | ReadStateUpdatedEvent;

export interface EventEnvelope {
  type: string;
  id: string;
  timestamp: string;
  payload: ServerEvent;
}

export interface TypingMessage {
  type: 'typing';
  conversation_id: string;
  conversation_type: 'channel' | 'direct_message';
}

export interface PingMessage {
  type: 'ping';
}

export type ClientMessage = TypingMessage | PingMessage;
