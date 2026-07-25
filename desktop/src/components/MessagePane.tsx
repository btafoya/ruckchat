import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import type { JSX } from 'react';
import { useParams, useSearchParams } from 'react-router-dom';
import {
  useChannelContext,
  useDirectMessageContext,
  useMessageContext,
  useOrgMemberContext,
  useOrganizationContext,
  useSessionContext,
  useSettingsContext,
  useTypingContext,
} from '../context';
import { useMarkReadBatcher } from '../hooks';
import { Composer } from './Composer';
import { MessageItem } from './MessageItem';
import { ThreadPane } from './ThreadPane';

export function MessagePane(): JSX.Element {
  const params = useParams<{
    organizationId?: string;
    channelId?: string;
    dmId?: string;
    messageId?: string;
  }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const { organizations } = useOrganizationContext();
  const { channels } = useChannelContext();
  const { conversations } = useDirectMessageContext();
  const { members: orgMembers } = useOrgMemberContext();
  const {
    messages,
    isLoading,
    isLoadingOlder,
    hasMoreOlder,
    hasMoreNewer,
    lastAppendedId,
    unreadIds,
    loadOlder,
    jumpToMessage,
    markRead,
  } = useMessageContext();
  const { typingUsers } = useTypingContext();

  const organization = organizations.find((o) => o.id === params.organizationId);
  const channel = channels.find((c) => c.id === params.channelId);
  const conversation = conversations.find((c) => c.id === params.dmId);
  const conversationType = params.channelId ? 'channel' : params.dmId ? 'direct_message' : undefined;
  const conversationId = params.channelId ?? params.dmId;

  const handleVisible = useMarkReadBatcher(apiUrl, session?.token, conversationType, conversationId, markRead);

  const scrollRef = useRef<HTMLDivElement | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);
  const topSentinelRef = useRef<HTMLDivElement | null>(null);
  const messageNodesRef = useRef<Map<string, HTMLLIElement>>(new Map());
  const isAtBottomRef = useRef(true);
  const [newMessageCount, setNewMessageCount] = useState(0);
  const scrollRestoreRef = useRef<{ scrollHeight: number; scrollTop: number } | null>(null);
  const didInitialScrollRef = useRef(false);
  const highlightId = searchParams.get('message');
  const [highlightedId, setHighlightedId] = useState<string | null>(null);
  const jumpedForRef = useRef<string | null>(null);

  const title = useMemo(() => {
    if (channel) {
      return `# ${channel.name}`;
    }
    if (conversation) {
      const others = conversation.member_ids.filter((id) => id !== session?.user.id);
      const names = others.map((id) => orgMembers.find((m) => m.id === id)?.display_name ?? id);
      return `DM: ${names.length > 0 ? names.join(', ') : 'You'}`;
    }
    return null;
  }, [channel, conversation, session?.user.id, orgMembers]);

  const typingList = useMemo(() => {
    if (!conversationId) {
      return [];
    }
    const users = typingUsers[conversationId] ?? [];
    return users.filter((id) => id !== session?.user.id);
  }, [conversationId, typingUsers, session?.user.id]);

  // Reset per-conversation scroll/pill bookkeeping when switching channels/DMs.
  useEffect(() => {
    isAtBottomRef.current = true;
    setNewMessageCount(0);
    didInitialScrollRef.current = false;
    jumpedForRef.current = null;
  }, [conversationId]);

  // Deep-link jump: load a window anchored on ?message=<id> once per id.
  useEffect(() => {
    if (!highlightId || !conversationId || jumpedForRef.current === highlightId) {
      return;
    }
    jumpedForRef.current = highlightId;
    didInitialScrollRef.current = true; // suppress the normal "scroll to bottom on open" behavior
    void jumpToMessage(highlightId);
  }, [highlightId, conversationId, jumpToMessage]);

  useEffect(() => {
    if (!highlightId || messages.length === 0) {
      return;
    }
    const node = messageNodesRef.current.get(highlightId);
    if (!node) {
      return;
    }
    node.scrollIntoView({ block: 'center' });
    setHighlightedId(highlightId);
    const timer = setTimeout(() => setHighlightedId(null), 2000);
    const { message: _discard, ...rest } = Object.fromEntries(searchParams);
    setSearchParams(rest, { replace: true });
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [highlightId, messages.length]);

  // Scroll to the newest message once, the first time a conversation's
  // history finishes loading (unless a deep-link jump already handled it).
  useEffect(() => {
    if (isLoading || didInitialScrollRef.current || messages.length === 0) {
      return;
    }
    didInitialScrollRef.current = true;
    bottomRef.current?.scrollIntoView({ block: 'end' });
  }, [isLoading, messages.length]);

  // Auto-follow vs. "N new messages" pill for genuinely new WebSocket
  // messages (as opposed to pagination-caused array changes).
  useEffect(() => {
    if (!lastAppendedId) {
      return;
    }
    if (isAtBottomRef.current) {
      bottomRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' });
    } else {
      setNewMessageCount((n) => n + 1);
    }
  }, [lastAppendedId]);

  // Track whether the bottom of the list is in view, to decide auto-follow.
  useEffect(() => {
    const node = bottomRef.current;
    const container = scrollRef.current;
    if (!node || !container || typeof IntersectionObserver === 'undefined') {
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        isAtBottomRef.current = entries[0]?.isIntersecting ?? false;
        if (isAtBottomRef.current) {
          setNewMessageCount(0);
        }
      },
      { root: container, threshold: 1 },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [conversationId]);

  const handleLoadOlder = useCallback(async () => {
    const container = scrollRef.current;
    if (container) {
      scrollRestoreRef.current = {
        scrollHeight: container.scrollHeight,
        scrollTop: container.scrollTop,
      };
    }
    await loadOlder();
  }, [loadOlder]);

  // Trigger loadOlder automatically as the top sentinel scrolls into view.
  useEffect(() => {
    const node = topSentinelRef.current;
    const container = scrollRef.current;
    if (!node || !container || typeof IntersectionObserver === 'undefined') {
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && hasMoreOlder && !isLoadingOlder) {
          void handleLoadOlder();
        }
      },
      { root: container, threshold: 0 },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [hasMoreOlder, isLoadingOlder, handleLoadOlder]);

  // Restore scroll position after older messages are prepended, so the
  // viewport doesn't visibly jump.
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
  }, [messages]);

  const jumpToLatest = useCallback(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' });
    setNewMessageCount(0);
  }, []);

  if (!organization) {
    return (
      <div className="flex flex-1 items-center justify-center bg-bg text-text-muted">
        Select an organization from the sidebar.
      </div>
    );
  }

  if (!conversationId || !conversationType || (!channel && !conversation)) {
    return (
      <div className="flex flex-1 items-center justify-center bg-bg text-text-muted">
        Select a channel or direct message in {organization.name}.
      </div>
    );
  }

  return (
    <section className="relative flex flex-1 flex-col overflow-hidden" aria-label="Messages">
      <header className="border-b border-border px-6 py-4">
        <h1 className="text-lg font-semibold text-text">{title}</h1>
        {channel?.topic && <p className="text-sm text-text-muted">{channel.topic}</p>}
      </header>

      <div ref={scrollRef} className="flex flex-1 flex-col overflow-y-auto p-4">
        <div ref={topSentinelRef} />
        {isLoadingOlder && (
          <div className="mb-3 self-center text-xs text-text-muted">Loading older messages...</div>
        )}

        {isLoading && messages.length === 0 && (
          <div className="text-text-muted">Loading messages...</div>
        )}
        {messages.length === 0 && !isLoading && (
          <div className="text-text-muted">No messages yet.</div>
        )}
        <ul className="flex flex-col gap-3">
          {messages.map((message) => (
            <li
              key={message.id}
              ref={(node) => {
                if (node) {
                  messageNodesRef.current.set(message.id, node);
                } else {
                  messageNodesRef.current.delete(message.id);
                }
              }}
              className={highlightedId === message.id ? 'rounded-md ring-2 ring-accent' : undefined}
            >
              <MessageItem
                message={message}
                organizationId={organization.id}
                isUnread={unreadIds.has(message.id) && message.author_id !== session?.user.id}
                onVisible={handleVisible}
              />
            </li>
          ))}
        </ul>
        <div ref={bottomRef} />

        {typingList.length > 0 && (
          <div className="mt-2 text-xs italic text-text-muted">
            {typingList.join(', ')} {typingList.length === 1 ? 'is' : 'are'} typing...
          </div>
        )}
      </div>

      {newMessageCount > 0 && (
        <button
          type="button"
          onClick={jumpToLatest}
          className="absolute bottom-24 left-1/2 -translate-x-1/2 rounded-full bg-accent px-4 py-1.5 text-sm font-semibold text-text-inverse shadow-lg hover:bg-accent-hover"
        >
          ↓ {newMessageCount} new {newMessageCount === 1 ? 'message' : 'messages'}
        </button>
      )}

      <Composer
        conversationType={conversationType}
        conversationId={conversationId}
        organizationId={organization.id}
      />

      {params.messageId && <ThreadPane />}
    </section>
  );
}
