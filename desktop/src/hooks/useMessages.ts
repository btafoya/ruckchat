import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { createApi } from '../api';
import type { Message, Reaction } from '../api';

const PAGE_SIZE = 50;
const MAX_PAGE_SIZE = 100;

export interface MessagesState {
  messages: Message[];
  isLoading: boolean;
  isLoadingMore: boolean;
  error: string | null;
  hasMore: boolean;
  refresh: () => Promise<void>;
  loadMore: () => Promise<void>;
  sendMessage: (content: string, parentId?: string, fileIds?: string[]) => Promise<Message | undefined>;
  retryMessage: (messageId: string) => Promise<void>;
  loadThreadReplies: (messageId: string) => Promise<void>;
  threadReplies: Message[];
  threadRepliesLoading: boolean;
  reactions: Record<string, Reaction[]>;
  addReaction: (messageId: string, reaction: Reaction) => void;
  removeReaction: (messageId: string, userId: string, emoji: string) => void;
  appendMessage: (message: Message) => void;
  updateMessage: (message: Message) => void;
  removeMessage: (messageId: string) => void;
}

export interface UseMessagesOptions {
  apiUrl?: string;
}

export function useMessages(
  token: string | undefined,
  conversationType: 'channel' | 'direct_message' | undefined,
  conversationId: string | undefined,
  userId: string | undefined,
  options: UseMessagesOptions = {},
  userDisplayName?: string,
): MessagesState {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [offset, setOffset] = useState(0);
  const [threadReplies, setThreadReplies] = useState<Message[]>([]);
  const [threadRepliesLoading, setThreadRepliesLoading] = useState(false);
  const [reactions, setReactions] = useState<Record<string, Reaction[]>>({});
  const api = useMemo(() => createApi(options.apiUrl), [options.apiUrl]);
  const pendingSendRef = useRef<Set<string>>(new Set());
  const pendingContentRef = useRef<Record<string, string>>({});

  const loadPage = useCallback(
    async (pageOffset: number, append: boolean) => {
      if (!token || !conversationType || !conversationId) {
        return [];
      }
      const items =
        conversationType === 'channel'
          ? await api.channels.listMessages(token, conversationId, PAGE_SIZE, pageOffset)
          : await api.directMessages.listMessages(token, conversationId, PAGE_SIZE, pageOffset);
      return items;
    },
    [api, token, conversationType, conversationId],
  );

  const refresh = useCallback(async () => {
    if (!token || !conversationType || !conversationId) {
      setMessages([]);
      setHasMore(true);
      setOffset(0);
      setReactions({});
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const items = await loadPage(0, false);
      setMessages(items);
      setOffset(items.length);
      setHasMore(items.length === PAGE_SIZE);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load messages');
    } finally {
      setIsLoading(false);
    }
  }, [loadPage, token, conversationType, conversationId]);

  const loadMore = useCallback(async () => {
    if (!token || !conversationType || !conversationId || isLoadingMore || !hasMore) {
      return;
    }
    setIsLoadingMore(true);
    try {
      const items = await loadPage(offset, true);
      setMessages((prev) => [...prev, ...items]);
      setOffset((prev) => prev + items.length);
      setHasMore(items.length === PAGE_SIZE);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more messages');
    } finally {
      setIsLoadingMore(false);
    }
  }, [loadPage, offset, hasMore, isLoadingMore, token, conversationType, conversationId]);

  const sendMessage = useCallback(
    async (content: string, parentId?: string, fileIds?: string[]): Promise<Message | undefined> => {
      if (!token || !conversationType || !conversationId || !userId) {
        return undefined;
      }
      const trimmed = content.trim();
      if (!trimmed) {
        return undefined;
      }

      const tempId = `pending-${Date.now()}`;
      pendingContentRef.current[tempId] = trimmed;
      const now = new Date().toISOString();
      const optimistic: Message = {
        id: tempId,
        conversation_id: conversationId,
        conversation_type: conversationType,
        parent_id: parentId,
        author_id: userId,
        author_display_name: userDisplayName ?? null,
        content: trimmed,
        mentioned_user_ids: [],
        created_at: now,
        updated_at: now,
        deleted_at: null,
      };
      pendingSendRef.current.add(tempId);
      appendMessage(optimistic);

      try {
        const request = { content: trimmed, parent_id: parentId };
        const posted =
          conversationType === 'channel'
            ? await api.channels.postMessage(token, conversationId, request)
            : await api.directMessages.postMessage(token, conversationId, request);

        for (const fileId of fileIds ?? []) {
          try {
            await api.files.attachToMessage(token, posted.id, fileId);
          } catch (attachErr) {
            console.warn('Failed to attach file', attachErr);
          }
        }

        setMessages((prev) => prev.map((m) => (m.id === tempId ? posted : m)));
        pendingSendRef.current.delete(tempId);
        delete pendingContentRef.current[tempId];
        return posted;
      } catch (err) {
        pendingSendRef.current.delete(tempId);
        setError(err instanceof Error ? err.message : 'Failed to send message');
        return undefined;
      }
    },
    [api, token, conversationType, conversationId, userId],
  );

  const retryMessage = useCallback(
    async (messageId: string) => {
      if (!token || !conversationType || !conversationId || !userId) {
        return;
      }
      const content = pendingContentRef.current[messageId];
      if (!content) {
        return;
      }
      const message = messages.find((m) => m.id === messageId);
      const parentId = message?.parent_id ?? undefined;

      setMessages((prev) => prev.filter((m) => m.id !== messageId));
      delete pendingContentRef.current[messageId];
      await sendMessage(content, parentId);
    },
    [messages, sendMessage, token, conversationType, conversationId, userId],
  );

  const loadThreadReplies = useCallback(
    async (messageId: string) => {
      if (!token) {
        return;
      }
      setThreadRepliesLoading(true);
      try {
        const items = await api.channels.listReplies(token, messageId, MAX_PAGE_SIZE, 0);
        setThreadReplies(items);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load thread replies');
      } finally {
        setThreadRepliesLoading(false);
      }
    },
    [api, token],
  );

  const appendMessage = useCallback((message: Message) => {
    setMessages((prev) => {
      if (prev.some((m) => m.id === message.id)) {
        return prev;
      }
      return [...prev, message];
    });
  }, []);

  const updateMessage = useCallback((message: Message) => {
    setMessages((prev) =>
      prev.map((m) => {
        if (m.id !== message.id) {
          return m;
        }
        return message;
      }),
    );
  }, []);

  const removeMessage = useCallback((messageId: string) => {
    setMessages((prev) => prev.filter((m) => m.id !== messageId));
  }, []);

  const addReaction = useCallback((messageId: string, reaction: Reaction) => {
    setReactions((prev) => {
      const list = prev[messageId] ?? [];
      if (list.some((r) => r.user_id === reaction.user_id && r.emoji === reaction.emoji)) {
        return prev;
      }
      return { ...prev, [messageId]: [...list, reaction] };
    });
  }, []);

  const removeReaction = useCallback((messageId: string, userId: string, emoji: string) => {
    setReactions((prev) => {
      const list = prev[messageId] ?? [];
      const filtered = list.filter((r) => !(r.user_id === userId && r.emoji === emoji));
      if (filtered.length === list.length) {
        return prev;
      }
      const next = { ...prev, [messageId]: filtered };
      if (next[messageId].length === 0) {
        delete next[messageId];
      }
      return next;
    });
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return useMemo(
    () => ({
      messages,
      isLoading,
      isLoadingMore,
      error,
      hasMore,
      refresh,
      loadMore,
      sendMessage,
      retryMessage,
      loadThreadReplies,
      threadReplies,
      threadRepliesLoading,
      reactions,
      addReaction,
      removeReaction,
      appendMessage,
      updateMessage,
      removeMessage,
    }),
    [
      messages,
      isLoading,
      isLoadingMore,
      error,
      hasMore,
      refresh,
      loadMore,
      sendMessage,
      retryMessage,
      loadThreadReplies,
      threadReplies,
      threadRepliesLoading,
      reactions,
      addReaction,
      removeReaction,
      appendMessage,
      updateMessage,
      removeMessage,
    ],
  );
}
