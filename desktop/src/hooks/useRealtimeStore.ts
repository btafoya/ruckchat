import { useCallback, useMemo } from 'react';
import type { ServerEvent } from '../api/events';
import type { MessagesState } from './useMessages';
import type { PresenceState } from './usePresence';
import type { TypingState } from './useTyping';

export interface RealtimeStore {
  onEvent: (event: ServerEvent) => void;
}

export function useRealtimeStore(
  messages: MessagesState,
  presence: PresenceState,
  typing: TypingState,
): RealtimeStore {
  const onEvent = useCallback(
    (event: ServerEvent) => {
      switch (event.type) {
        case 'message.created':
          messages.appendMessage(event.message);
          break;
        case 'message.updated':
          messages.updateMessage(event.message);
          break;
        case 'message.deleted':
          messages.removeMessage(event.message.id);
          break;
        case 'reaction.added':
          // Reactions are applied to messages in the messaging task.
          break;
        case 'reaction.removed':
          // Reactions are applied to messages in the messaging task.
          break;
        case 'typing.updated':
          typing.addTypingUser(event.conversation_id, event.user_id);
          break;
        case 'presence.updated':
          presence.setUserPresence(event.user_id, event.status);
          break;
        case 'connection.established':
          // Handled by connection status UI if needed.
          break;
      }
    },
    [messages, presence, typing],
  );

  return useMemo(
    () => ({
      onEvent,
    }),
    [onEvent],
  );
}
