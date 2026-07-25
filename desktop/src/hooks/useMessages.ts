import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { createApi } from '../api';
import type { Message, MessagePageResponse, Reaction } from '../api';

const PAGE_SIZE = 50;

export interface MessagesState {
  messages: Message[];
  isLoading: boolean;
  isLoadingOlder: boolean;
  isLoadingNewer: boolean;
  error: string | null;
  hasMoreOlder: boolean;
  hasMoreNewer: boolean;
  unreadIds: Set<string>;
  /** Id of the most recent message appended via a live WebSocket event,
   * distinct from pagination-caused array changes. Consumers use this to
   * decide whether to auto-scroll or show a "new messages" indicator. */
  lastAppendedId: string | null;
  refresh: () => Promise<void>;
  loadOlder: () => Promise<void>;
  loadNewer: () => Promise<void>;
  jumpToMessage: (messageId: string) => Promise<void>;
  sendMessage: (content: string, parentId?: string, fileIds?: string[]) => Promise<Message | undefined>;
  retryMessage: (messageId: string) => Promise<void>;
  loadThreadReplies: (messageId: string) => Promise<void>;
  loadOlderReplies: (messageId: string) => Promise<void>;
  loadNewerReplies: (messageId: string) => Promise<void>;
  threadReplies: Message[];
  threadRepliesLoading: boolean;
  threadHasMoreOlder: boolean;
  threadHasMoreNewer: boolean;
  reactions: Record<string, Reaction[]>;
  addReaction: (messageId: string, reaction: Reaction) => void;
  removeReaction: (messageId: string, userId: string, emoji: string) => void;
  appendMessage: (message: Message) => void;
  updateMessage: (message: Message) => void;
  removeMessage: (messageId: string) => void;
  markRead: (messageId: string) => void;
  editingMessage: Message | null;
  startEdit: (message: Message) => void;
  cancelEdit: () => void;
  saveEdit: (content: string) => Promise<void>;
}

export interface UseMessagesOptions {
  apiUrl?: string;
}

function unreadIdsOf(items: MessagePageResponse['items']): string[] {
  return items.filter((item) => item.is_unread).map((item) => item.id);
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
  const [isLoadingOlder, setIsLoadingOlder] = useState(false);
  const [isLoadingNewer, setIsLoadingNewer] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasMoreOlder, setHasMoreOlder] = useState(true);
  // hasMoreNewer doubles as the "anchored-history" flag: while true, we
  // haven't paginated forward to the real conversation tail yet, so
  // WebSocket-delivered messages must not be spliced in (see ADR-016).
  const [hasMoreNewer, setHasMoreNewer] = useState(false);
  const [unreadIds, setUnreadIds] = useState<Set<string>>(new Set());
  const [lastAppendedId, setLastAppendedId] = useState<string | null>(null);
  const [threadReplies, setThreadReplies] = useState<Message[]>([]);
  const [threadRepliesLoading, setThreadRepliesLoading] = useState(false);
  const [threadHasMoreOlder, setThreadHasMoreOlder] = useState(true);
  const [threadHasMoreNewer, setThreadHasMoreNewer] = useState(false);
  const [reactions, setReactions] = useState<Record<string, Reaction[]>>({});
  const [editingMessage, setEditingMessage] = useState<Message | null>(null);
  const api = useMemo(() => createApi(options.apiUrl), [options.apiUrl]);
  const pendingSendRef = useRef<Set<string>>(new Set());
  const pendingContentRef = useRef<Record<string, string>>({});

  const listPage = useCallback(
    async (query: { beforeId?: string; afterId?: string; aroundId?: string }) => {
      if (!token || !conversationType || !conversationId) {
        return null;
      }
      const pageQuery = { ...query, limit: PAGE_SIZE };
      return conversationType === 'channel'
        ? await api.channels.listMessages(token, conversationId, pageQuery)
        : await api.directMessages.listMessages(token, conversationId, pageQuery);
    },
    [api, token, conversationType, conversationId],
  );

  const mergeUnread = useCallback((items: MessagePageResponse['items']) => {
    const newlyUnread = unreadIdsOf(items);
    if (newlyUnread.length === 0) {
      return;
    }
    setUnreadIds((prev) => new Set([...prev, ...newlyUnread]));
  }, []);

  const refresh = useCallback(async () => {
    if (!token || !conversationType || !conversationId) {
      setMessages([]);
      setHasMoreOlder(true);
      setHasMoreNewer(false);
      setUnreadIds(new Set());
      setReactions({});
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const page = await listPage({});
      if (!page) {
        return;
      }
      setMessages(page.items);
      setHasMoreOlder(page.has_more_older);
      setHasMoreNewer(false);
      setUnreadIds(new Set(unreadIdsOf(page.items)));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load messages');
    } finally {
      setIsLoading(false);
    }
  }, [listPage, token, conversationType, conversationId]);

  const loadOlder = useCallback(async () => {
    if (!hasMoreOlder || isLoadingOlder || messages.length === 0) {
      return;
    }
    setIsLoadingOlder(true);
    try {
      const page = await listPage({ beforeId: messages[0].id });
      if (!page) {
        return;
      }
      setMessages((prev) => [...page.items, ...prev]);
      setHasMoreOlder(page.has_more_older);
      mergeUnread(page.items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load older messages');
    } finally {
      setIsLoadingOlder(false);
    }
  }, [listPage, mergeUnread, hasMoreOlder, isLoadingOlder, messages]);

  const loadNewer = useCallback(async () => {
    if (!hasMoreNewer || isLoadingNewer || messages.length === 0) {
      return;
    }
    setIsLoadingNewer(true);
    try {
      const page = await listPage({ afterId: messages[messages.length - 1].id });
      if (!page) {
        return;
      }
      setMessages((prev) => [...prev, ...page.items]);
      setHasMoreNewer(page.has_more_newer);
      mergeUnread(page.items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load newer messages');
    } finally {
      setIsLoadingNewer(false);
    }
  }, [listPage, mergeUnread, hasMoreNewer, isLoadingNewer, messages]);

  const jumpToMessage = useCallback(
    async (messageId: string) => {
      setIsLoading(true);
      setError(null);
      try {
        const page = await listPage({ aroundId: messageId });
        if (!page) {
          return;
        }
        setMessages(page.items);
        setHasMoreOlder(page.has_more_older);
        setHasMoreNewer(page.has_more_newer);
        setUnreadIds(new Set(unreadIdsOf(page.items)));
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load message');
      } finally {
        setIsLoading(false);
      }
    },
    [listPage],
  );

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
      setMessages((prev) => [...prev, optimistic]);

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
    [api, token, conversationType, conversationId, userId, userDisplayName],
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

  const listRepliesPage = useCallback(
    async (messageId: string, query: { beforeId?: string; afterId?: string; aroundId?: string }) => {
      if (!token) {
        return null;
      }
      return api.channels.listReplies(token, messageId, { ...query, limit: PAGE_SIZE });
    },
    [api, token],
  );

  const loadThreadReplies = useCallback(
    async (messageId: string) => {
      setThreadRepliesLoading(true);
      try {
        const page = await listRepliesPage(messageId, {});
        if (!page) {
          return;
        }
        setThreadReplies(page.items);
        setThreadHasMoreOlder(page.has_more_older);
        setThreadHasMoreNewer(false);
        mergeUnread(page.items);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load thread replies');
      } finally {
        setThreadRepliesLoading(false);
      }
    },
    [listRepliesPage, mergeUnread],
  );

  const loadOlderReplies = useCallback(
    async (messageId: string) => {
      if (!threadHasMoreOlder || threadRepliesLoading || threadReplies.length === 0) {
        return;
      }
      setThreadRepliesLoading(true);
      try {
        const page = await listRepliesPage(messageId, { beforeId: threadReplies[0].id });
        if (!page) {
          return;
        }
        setThreadReplies((prev) => [...page.items, ...prev]);
        setThreadHasMoreOlder(page.has_more_older);
        mergeUnread(page.items);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load older replies');
      } finally {
        setThreadRepliesLoading(false);
      }
    },
    [listRepliesPage, mergeUnread, threadHasMoreOlder, threadRepliesLoading, threadReplies],
  );

  const loadNewerReplies = useCallback(
    async (messageId: string) => {
      if (!threadHasMoreNewer || threadRepliesLoading || threadReplies.length === 0) {
        return;
      }
      setThreadRepliesLoading(true);
      try {
        const page = await listRepliesPage(messageId, {
          afterId: threadReplies[threadReplies.length - 1].id,
        });
        if (!page) {
          return;
        }
        setThreadReplies((prev) => [...prev, ...page.items]);
        setThreadHasMoreNewer(page.has_more_newer);
        mergeUnread(page.items);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load newer replies');
      } finally {
        setThreadRepliesLoading(false);
      }
    },
    [listRepliesPage, mergeUnread, threadHasMoreNewer, threadRepliesLoading, threadReplies],
  );

  const appendMessage = useCallback(
    (message: Message) => {
      if (message.conversation_id !== conversationId) {
        // The WS connection is shared across every conversation the user
        // belongs to (server auto-subscribes per organization/DM); only
        // splice in messages for the conversation this hook instance is
        // actually bound to, or they'd render in whichever conversation
        // happens to be open.
        return;
      }
      if (hasMoreNewer) {
        // Anchored-history mode: we haven't paginated forward to the real
        // tail yet, so splicing a WS-delivered message in here would leave
        // an undetectable gap. Dropped; loadNewer() will pick it up in its
        // turn once the user scrolls down that far.
        return;
      }
      setMessages((prev) => {
        if (prev.some((m) => m.id === message.id)) {
          return prev;
        }
        return [...prev, message];
      });
      setLastAppendedId(message.id);
    },
    [conversationId, hasMoreNewer],
  );

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

  const markRead = useCallback((messageId: string) => {
    setUnreadIds((prev) => {
      if (!prev.has(messageId)) {
        return prev;
      }
      const next = new Set(prev);
      next.delete(messageId);
      return next;
    });
  }, []);

  const startEdit = useCallback((message: Message) => {
    setEditingMessage(message);
  }, []);

  const cancelEdit = useCallback(() => {
    setEditingMessage(null);
  }, []);

  const saveEdit = useCallback(
    async (content: string): Promise<void> => {
      if (!token || !editingMessage) {
        return;
      }
      const trimmed = content.trim();
      if (!trimmed) {
        return;
      }
      try {
        const updated = await api.messages.edit(token, editingMessage.id, trimmed);
        updateMessage(updated);
        setEditingMessage(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to edit message');
      }
    },
    [api, token, editingMessage, updateMessage],
  );

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
      isLoadingOlder,
      isLoadingNewer,
      error,
      hasMoreOlder,
      hasMoreNewer,
      unreadIds,
      lastAppendedId,
      refresh,
      loadOlder,
      loadNewer,
      jumpToMessage,
      sendMessage,
      retryMessage,
      loadThreadReplies,
      loadOlderReplies,
      loadNewerReplies,
      threadReplies,
      threadRepliesLoading,
      threadHasMoreOlder,
      threadHasMoreNewer,
      reactions,
      addReaction,
      removeReaction,
      appendMessage,
      updateMessage,
      removeMessage,
      markRead,
      editingMessage,
      startEdit,
      cancelEdit,
      saveEdit,
    }),
    [
      messages,
      isLoading,
      isLoadingOlder,
      isLoadingNewer,
      error,
      hasMoreOlder,
      hasMoreNewer,
      unreadIds,
      lastAppendedId,
      refresh,
      loadOlder,
      loadNewer,
      jumpToMessage,
      sendMessage,
      retryMessage,
      loadThreadReplies,
      loadOlderReplies,
      loadNewerReplies,
      threadReplies,
      threadRepliesLoading,
      threadHasMoreOlder,
      threadHasMoreNewer,
      reactions,
      addReaction,
      removeReaction,
      appendMessage,
      updateMessage,
      removeMessage,
      markRead,
      editingMessage,
      startEdit,
      cancelEdit,
      saveEdit,
    ],
  );
}
