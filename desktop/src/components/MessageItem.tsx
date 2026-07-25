import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { JSX } from 'react';
import { NavLink } from 'react-router-dom';
import { createApi } from '../api';
import type { Message } from '../api';
import { useMessageContext, useSessionContext, useSettingsContext } from '../context';
import { MessageContent } from './MessageContent';

const QUICK_REACTIONS = ['👍', '❤️', '😂', '😮', '😢', '🎉'];

interface MessageItemProps {
  message: Message;
  organizationId: string;
  showReplyButton?: boolean;
  /** Whether this message is unread by the caller; renders a dot marker. */
  isUnread?: boolean;
  /** Called once, the instant this message scrolls into view, when unread. */
  onVisible?: (messageId: string) => void;
}

export function MessageItem({
  message,
  organizationId,
  showReplyButton = true,
  isUnread = false,
  onVisible,
}: MessageItemProps): JSX.Element {
  const { session } = useSessionContext();
  const { reactions, addReaction, removeReaction, retryMessage, startEdit, deleteMessage } = useMessageContext();
  const [isConfirmingDelete, setIsConfirmingDelete] = useState(false);
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [isReacting, setIsReacting] = useState(false);
  const rootRef = useRef<HTMLElement | null>(null);
  const notifiedRef = useRef(false);

  useEffect(() => {
    if (!isUnread || !onVisible || notifiedRef.current) {
      return;
    }
    const node = rootRef.current;
    if (!node || typeof IntersectionObserver === 'undefined') {
      return;
    }
    // Without an explicit `root`, IntersectionObserver checks intersection
    // against the top-level document viewport, not the actual scrollable
    // message list — so items clipped by the list's own `overflow-y-auto`
    // would still report as "intersecting" (the container itself is
    // on-screen), marking everything read on mount. Anchor to the nearest
    // scrollable ancestor instead.
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && !notifiedRef.current) {
          notifiedRef.current = true;
          onVisible(message.id);
          observer.disconnect();
        }
      },
      { root: node.closest('.overflow-y-auto'), threshold: 0.5 },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [isUnread, onVisible, message.id]);

  const messageReactions = reactions[message.id] ?? [];
  const isDeleted = message.deleted_at != null;
  const isPending = message.id.startsWith('pending-');
  const canEdit = !isDeleted && !isPending && message.author_id === session?.user.id;
  const canDelete = !isDeleted && !isPending && message.author_id === session?.user.id;

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

  const confirmDeleteTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (confirmDeleteTimeoutRef.current) {
        clearTimeout(confirmDeleteTimeoutRef.current);
      }
    };
  }, []);

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

  const handleDelete = useCallback(() => {
    if (isConfirmingDelete) {
      if (confirmDeleteTimeoutRef.current) {
        clearTimeout(confirmDeleteTimeoutRef.current);
        confirmDeleteTimeoutRef.current = null;
      }
      void deleteMessage(message.id);
      setIsConfirmingDelete(false);
      return;
    }
    setIsConfirmingDelete(true);
    confirmDeleteTimeoutRef.current = setTimeout(() => {
      setIsConfirmingDelete(false);
    }, 3000);
  }, [isConfirmingDelete, deleteMessage, message.id]);

  const replyPath =
    message.conversation_type === 'channel'
      ? `/org/${organizationId}/channel/${message.conversation_id}/thread/${message.id}`
      : `/org/${organizationId}/dm/${message.conversation_id}/thread/${message.id}`;

  return (
    <article ref={rootRef} className="flex flex-col gap-1 rounded-md p-2 hover:bg-message-hover">
      <div className="flex items-baseline gap-2">
        {isUnread && (
          <span
            className="h-2 w-2 flex-shrink-0 rounded-full bg-accent"
            aria-label="Unread message"
          />
        )}
        <span className="text-sm font-semibold text-accent">{message.author_display_name ?? message.author_id}</span>
        <span className="text-xs text-text-muted">{new Date(message.created_at).toLocaleString()}</span>
        {isPending && <span className="text-xs text-warning">Sending...</span>}
      </div>
      <div className="text-sm text-text">
        {isDeleted ? (
          <span className="italic text-text-muted">[deleted]</span>
        ) : (
          <MessageContent content={message.content} />
        )}
      </div>

      {!isDeleted && message.attachments && message.attachments.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {message.attachments.map((file) => (
            <a
              key={file.id}
              href={`${apiUrl}/files/${file.id}/content`}
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-1 rounded-full bg-surface-elevated px-2 py-1 text-xs text-text hover:underline"
            >
              📎 {file.file_name}
            </a>
          ))}
        </div>
      )}

      {grouped.length > 0 && (
        <div className="mt-1 flex flex-wrap items-center gap-1">
          {grouped.map(([emoji, { count, hasMe }]) => (
            <button
              key={emoji}
              type="button"
              onClick={() => void toggleReaction(emoji)}
              disabled={isReacting}
              className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs ${
                hasMe ? 'bg-accent-bg text-text' : 'bg-surface-elevated text-text'
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
            className="rounded-md px-1 py-0.5 text-sm text-text-muted hover:bg-surface-elevated hover:text-text disabled:opacity-50"
            aria-label={`React with ${emoji}`}
          >
            {emoji}
          </button>
        ))}
        {showReplyButton && !isPending && (
          <NavLink
            to={replyPath}
            className="ml-2 text-xs text-text-muted hover:text-text"
          >
            Reply in thread
          </NavLink>
        )}
        {canEdit && (
          <button
            type="button"
            onClick={() => startEdit(message)}
            className="ml-2 text-xs text-text-muted hover:text-text"
          >
            Edit
          </button>
        )}
        {canDelete && (
          <button
            type="button"
            onClick={() => void handleDelete()}
            className={`ml-2 text-xs ${isConfirmingDelete ? 'text-danger hover:text-danger-hover' : 'text-text-muted hover:text-text'}`}
          >
            {isConfirmingDelete ? 'Confirm Delete' : 'Delete'}
          </button>
        )}
        {isPending && (
          <button
            type="button"
            onClick={() => void retryMessage(message.id)}
            className="ml-2 text-xs text-warning hover:text-warning-hover"
          >
            Retry
          </button>
        )}
      </div>
    </article>
  );
}
