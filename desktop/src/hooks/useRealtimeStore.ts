import { useCallback, useMemo } from 'react';
import type { ServerEvent } from '../api/events';
import type { MessagesState } from './useMessages';
import type { NotificationState } from './useNotifications';
import type { PresenceState } from './usePresence';
import type { TypingState } from './useTyping';
import type { UnreadState } from './useUnread';

export interface RealtimeStore {
  onEvent: (event: ServerEvent) => void;
}

export function useRealtimeStore(
  messages: MessagesState,
  presence: PresenceState,
  typing: TypingState,
  unread: UnreadState,
  notifications?: NotificationState,
): RealtimeStore {
  const onEvent = useCallback(
    (event: ServerEvent) => {
      switch (event.type) {
        case 'message.created':
          messages.appendMessage(event.message);
          unread.increment(event.message.conversation_id);
          void notifications?.maybeNotify(event);
          break;
        case 'message.updated':
          messages.updateMessage(event.message);
          break;
        case 'message.deleted':
          messages.removeMessage(event.message.id);
          break;
        case 'reaction.added':
          messages.addReaction(event.message_id, {
            message_id: event.message_id,
            user_id: event.user_id,
            emoji: event.emoji,
            created_at: new Date().toISOString(),
          });
          break;
        case 'reaction.removed':
          messages.removeReaction(event.message_id, event.user_id, event.emoji);
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
    [messages, presence, typing, unread, notifications],
  );

  return useMemo(
    () => ({
      onEvent,
    }),
    [onEvent],
  );
}
