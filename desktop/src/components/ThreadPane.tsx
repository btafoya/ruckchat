import { useCallback, useEffect, useLayoutEffect, useMemo, useRef } from 'react';
import type { JSX } from 'react';
import { NavLink, useParams } from 'react-router-dom';
import { useMessageContext, useSessionContext, useSettingsContext } from '../context';
import { useMarkReadBatcher } from '../hooks';
import { Composer } from './Composer';
import { MessageItem } from './MessageItem';

export function ThreadPane(): JSX.Element {
  const params = useParams<{
    organizationId?: string;
    channelId?: string;
    dmId?: string;
    messageId?: string;
  }>();
  const organizationId = params.organizationId;
  const channelId = params.channelId;
  const dmId = params.dmId;
  const messageId = params.messageId;
  const conversationType = channelId ? 'channel' : 'direct_message';
  const conversationId = channelId ?? dmId;
  const backPath = channelId
    ? `/org/${organizationId}/channel/${channelId}`
    : `/org/${organizationId}/dm/${dmId}`;

  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const {
    messages,
    threadReplies,
    threadRepliesLoading,
    threadHasMoreOlder,
    unreadIds,
    loadThreadReplies,
    loadOlderReplies,
    markRead,
  } = useMessageContext();

  const handleVisible = useMarkReadBatcher(apiUrl, session?.token, conversationType, conversationId, markRead);

  const scrollRef = useRef<HTMLDivElement | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);
  const topSentinelRef = useRef<HTMLDivElement | null>(null);
  const scrollRestoreRef = useRef<{ scrollHeight: number; scrollTop: number } | null>(null);
  const didInitialScrollRef = useRef(false);

  useEffect(() => {
    didInitialScrollRef.current = false;
    if (messageId) {
      void loadThreadReplies(messageId);
    }
  }, [loadThreadReplies, messageId]);

  useEffect(() => {
    if (threadRepliesLoading || didInitialScrollRef.current || threadReplies.length === 0) {
      return;
    }
    didInitialScrollRef.current = true;
    bottomRef.current?.scrollIntoView({ block: 'end' });
  }, [threadRepliesLoading, threadReplies.length]);

  const handleLoadOlder = useCallback(async () => {
    if (!messageId) {
      return;
    }
    const container = scrollRef.current;
    if (container) {
      scrollRestoreRef.current = {
        scrollHeight: container.scrollHeight,
        scrollTop: container.scrollTop,
      };
    }
    await loadOlderReplies(messageId);
  }, [loadOlderReplies, messageId]);

  useEffect(() => {
    const node = topSentinelRef.current;
    const container = scrollRef.current;
    if (!node || !container || typeof IntersectionObserver === 'undefined') {
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && threadHasMoreOlder && !threadRepliesLoading) {
          void handleLoadOlder();
        }
      },
      { root: container, threshold: 0 },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [threadHasMoreOlder, threadRepliesLoading, handleLoadOlder]);

  useLayoutEffect(() => {
    const container = scrollRef.current;
    const restore = scrollRestoreRef.current;
    if (!container || !restore) {
      return;
    }
    scrollRestoreRef.current = null;
    const delta = container.scrollHeight - restore.scrollHeight;
    if (delta > 0) {
      container.scrollTop = restore.scrollTop + delta;
    }
  }, [threadReplies]);

  const parent = useMemo(
    () => messages.find((m) => m.id === messageId),
    [messages, messageId],
  );

  if (!organizationId || !conversationId || !messageId) {
    return <div />;
  }

  return (
    <div className="absolute inset-0 z-20 flex justify-end bg-overlay">
      <section className="flex w-full max-w-md flex-col border-l border-border bg-surface shadow-xl">
        <header className="flex items-center justify-between border-b border-border px-4 py-3">
          <div className="text-sm font-semibold text-text">Thread</div>
          <NavLink
            to={backPath}
            className="text-sm text-text hover:text-text-muted"
          >
            Close
          </NavLink>
        </header>

        <div ref={scrollRef} className="flex flex-1 flex-col overflow-y-auto p-4">
          {parent && <MessageItem message={parent} organizationId={organizationId} showReplyButton={false} />}
          {!parent && !threadRepliesLoading && (
            <div className="text-sm text-text-muted">Parent message not found.</div>
          )}

          <div className="my-2 border-t border-border" />

          <div ref={topSentinelRef} />
          {threadRepliesLoading && <div className="text-sm text-text-muted">Loading replies...</div>}
          {threadReplies.length === 0 && !threadRepliesLoading && (
            <div className="text-sm text-text-muted">No replies yet.</div>
          )}
          <ul className="flex flex-col gap-3">
            {threadReplies.map((reply) => (
              <li key={reply.id}>
                <MessageItem
                  message={reply}
                  organizationId={organizationId}
                  showReplyButton={false}
                  isUnread={unreadIds.has(reply.id) && reply.author_id !== session?.user.id}
                  onVisible={handleVisible}
                />
              </li>
            ))}
          </ul>
          <div ref={bottomRef} />
        </div>

        <Composer
          conversationType={conversationType}
          conversationId={conversationId}
          organizationId={organizationId}
          parentId={messageId}
          placeholder="Reply in thread..."
        />
      </section>
    </div>
  );
}
