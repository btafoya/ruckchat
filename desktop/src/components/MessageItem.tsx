import { useCallback, useMemo, useState } from 'react';
import type { JSX } from 'react';
import { NavLink } from 'react-router-dom';
import { createApi } from '../api';
import type { Message } from '../api';
import { useMessageContext, useSessionContext } from '../context';

const QUICK_REACTIONS = ['👍', '❤️', '😂', '😮', '😢', '🎉'];

interface MessageItemProps {
  message: Message;
  organizationId: string;
  showReplyButton?: boolean;
}

export function MessageItem({ message, organizationId, showReplyButton = true }: MessageItemProps): JSX.Element {
  const { session } = useSessionContext();
  const { reactions, addReaction, removeReaction, retryMessage } = useMessageContext();
  const api = useMemo(() => createApi(), []);
  const [isReacting, setIsReacting] = useState(false);

  const messageReactions = reactions[message.id] ?? [];
  const isDeleted = message.deleted_at != null;
  const isPending = message.id.startsWith('pending-');

  const grouped = useMemo(() => {
    const map = new Map<string, { count: number; hasMe: boolean }>();
    for (const reaction of messageReactions) {
      const existing = map.get(reaction.emoji);
      const isMe = reaction.user_id === session?.user.id;
      if (existing) {
        existing.count += 1;
        existing.hasMe = existing.hasMe || isMe;
      } else {
        map.set(reaction.emoji, { count: 1, hasMe: isMe });
      }
    }
    return Array.from(map.entries());
  }, [messageReactions, session?.user.id]);

  const toggleReaction = useCallback(
    async (emoji: string) => {
      if (!session || isReacting) {
        return;
      }
      const hasReacted = messageReactions.some(
        (r) => r.user_id === session.user.id && r.emoji === emoji,
      );
      setIsReacting(true);
      try {
        if (hasReacted) {
          await api.reactions.remove(session.token, message.id, emoji);
          removeReaction(message.id, session.user.id, emoji);
        } else {
          const reaction = await api.reactions.add(session.token, message.id, emoji);
          addReaction(message.id, reaction);
        }
      } catch (err) {
        console.warn('Failed to toggle reaction', err);
      } finally {
        setIsReacting(false);
      }
    },
    [addReaction, api, isReacting, message.id, messageReactions, removeReaction, session],
  );

  const replyPath =
    message.conversation_type === 'channel'
      ? `/org/${organizationId}/channel/${message.conversation_id}/thread/${message.id}`
      : `/org/${organizationId}/dm/${message.conversation_id}/thread/${message.id}`;

  return (
    <article className="flex flex-col gap-1 rounded-md p-2 hover:bg-gray-800/50">
      <div className="flex items-baseline gap-2">
        <span className="text-sm font-semibold text-green-400">{message.author_display_name ?? message.author_id}</span>
        <span className="text-xs text-gray-500">{new Date(message.created_at).toLocaleString()}</span>
        {isPending && <span className="text-xs text-yellow-500">Sending...</span>}
      </div>
      <div className="whitespace-pre-wrap text-sm text-gray-200">
        {isDeleted ? <span className="italic text-gray-500">[deleted]</span> : message.content}
      </div>

      {grouped.length > 0 && (
        <div className="mt-1 flex flex-wrap items-center gap-1">
          {grouped.map(([emoji, { count, hasMe }]) => (
            <button
              key={emoji}
              type="button"
              onClick={() => void toggleReaction(emoji)}
              disabled={isReacting}
              className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs ${
                hasMe ? 'bg-green-700/40 text-green-100' : 'bg-gray-700 text-gray-200'
              }`}
            >
              <span>{emoji}</span>
              {count > 1 && <span>{count}</span>}
            </button>
          ))}
        </div>
      )}

      <div className="mt-1 flex items-center gap-1">
        {QUICK_REACTIONS.map((emoji) => (
          <button
            key={emoji}
            type="button"
            onClick={() => void toggleReaction(emoji)}
            disabled={isReacting || isPending}
            className="rounded-md px-1 py-0.5 text-sm text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-50"
            aria-label={`React with ${emoji}`}
          >
            {emoji}
          </button>
        ))}
        {showReplyButton && !isPending && (
          <NavLink
            to={replyPath}
            className="ml-2 text-xs text-gray-400 hover:text-white"
          >
            Reply in thread
          </NavLink>
        )}
        {isPending && (
          <button
            type="button"
            onClick={() => void retryMessage(message.id)}
            className="ml-2 text-xs text-yellow-400 hover:text-yellow-300"
          >
            Retry
          </button>
        )}
      </div>
    </article>
  );
}
