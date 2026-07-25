import { useCallback, useMemo } from 'react';
import type { ServerEvent } from '../api/events';
import type { MessagesState } from './useMessages';
import type { NotificationState } from './useNotifications';
import type { PresenceState } from './usePresence';
import type { TypingState } from './useTyping';
import type { ReadState } from './useReadState';

export interface RealtimeStore {
  onEvent: (event: ServerEvent) => void;
}

export function useRealtimeStore(
  messages: MessagesState,
  presence: PresenceState,
  typing: TypingState,
  readState: ReadState,
  notifications?: NotificationState,
): RealtimeStore {
  const onEvent = useCallback(
    (event: ServerEvent) => {
      switch (event.type) {
        case 'message_created':
          messages.appendMessage(event.message);
          readState.increment(event.message.conversation_id);
          void notifications?.maybeNotify(event);
          break;
        case 'message_updated':
          messages.updateMessage(event.message);
          break;
        case 'message_deleted':
          messages.removeMessage(event.message.id);
          break;
        case 'reaction_added':
          messages.addReaction(event.message_id, {
            message_id: event.message_id,
            user_id: event.user_id,
            emoji: event.emoji,
            created_at: new Date().toISOString(),
          });
          break;
        case 'reaction_removed':
          messages.removeReaction(event.message_id, event.user_id, event.emoji);
          break;
        case 'typing':
          typing.addTypingUser(event.conversation_id, event.user_id);
          break;
        case 'presence':
          presence.setUserPresence(event.user_id, event.status);
          break;
        case 'connection_established':
          // Handled by connection status UI if needed.
          break;
        case 'read_state_updated':
          readState.applyRemoteRead(event.conversation_id);
          for (const messageId of event.message_ids) {
            messages.markRead(messageId);
          }
          break;
      }
    },
    [messages, presence, typing, readState, notifications],
  );

  return useMemo(
    () => ({
      onEvent,
    }),
    [onEvent],
  );
}
