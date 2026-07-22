import type { Message } from '../api/types';

export type PresenceStatus = 'online' | 'offline';

export interface MessageCreatedEvent {
  type: 'message.created';
  message: Message;
}

export interface MessageUpdatedEvent {
  type: 'message.updated';
  message: Message;
}

export interface MessageDeletedEvent {
  type: 'message.deleted';
  message: Message;
}

export interface ReactionAddedEvent {
  type: 'reaction.added';
  message_id: string;
  user_id: string;
  emoji: string;
}

export interface ReactionRemovedEvent {
  type: 'reaction.removed';
  message_id: string;
  user_id: string;
  emoji: string;
}

export interface TypingEvent {
  type: 'typing.updated';
  user_id: string;
  conversation_id: string;
  conversation_type: 'channel' | 'direct_message';
}

export interface PresenceEvent {
  type: 'presence.updated';
  user_id: string;
  status: PresenceStatus;
}

export interface ConnectionEstablishedEvent {
  type: 'connection.established';
  user_id: string;
}

export type ServerEvent =
  | MessageCreatedEvent
  | MessageUpdatedEvent
  | MessageDeletedEvent
  | ReactionAddedEvent
  | ReactionRemovedEvent
  | TypingEvent
  | PresenceEvent
  | ConnectionEstablishedEvent;

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
