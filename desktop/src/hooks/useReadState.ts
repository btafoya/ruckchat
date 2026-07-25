import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi } from '../api';

export interface ReadState {
  counts: Record<string, number>;
  total: number;
  increment: (conversationId: string) => void;
  markRead: (
    conversationId: string,
    conversationType: 'channel' | 'direct_message',
    messageIds: string[],
  ) => Promise<void>;
  /** Applies a read-state update received from one of the user's own other sessions. */
  applyRemoteRead: (conversationId: string) => void;
  refresh: () => Promise<void>;
}

export interface UseReadStateOptions {
  apiUrl?: string;
}

/** Server-side per-message read tracking, replacing the old localStorage-only unread badges. */
export function useReadState(
  token: string | undefined,
  organizationId: string | undefined,
  activeConversationId: string | undefined,
  options: UseReadStateOptions = {},
): ReadState {
  const [counts, setCounts] = useState<Record<string, number>>({});
  const api = useMemo(() => createApi(options.apiUrl), [options.apiUrl]);

  const refresh = useCallback(async () => {
    if (!token || !organizationId) {
      setCounts({});
      return;
    }
    try {
      const response = await api.organizations.unreadCounts(token, organizationId);
      setCounts(response.counts as unknown as Record<string, number>);
    } catch {
      // ignore transient failures; the next refresh retries
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const increment = useCallback(
    (conversationId: string) => {
      if (conversationId === activeConversationId) {
        return;
      }
      setCounts((prev) => ({ ...prev, [conversationId]: (prev[conversationId] ?? 0) + 1 }));
    },
    [activeConversationId],
  );

  const clearCount = useCallback((conversationId: string) => {
    setCounts((prev) => {
      if (!prev[conversationId]) {
        return prev;
      }
      const next = { ...prev };
      delete next[conversationId];
      return next;
    });
  }, []);

  const markRead = useCallback(
    async (
      conversationId: string,
      conversationType: 'channel' | 'direct_message',
      messageIds: string[],
    ) => {
      clearCount(conversationId);
      if (!token || messageIds.length === 0) {
        return;
      }
      try {
        if (conversationType === 'channel') {
          await api.channels.markRead(token, conversationId, messageIds);
        } else {
          await api.directMessages.markRead(token, conversationId, messageIds);
        }
      } catch {
        // best-effort; the next refresh reconciles the count
      }
    },
    [api, token, clearCount],
  );

  return useMemo(
    () => ({
      counts,
      total: Object.values(counts).reduce((sum, c) => sum + c, 0),
      increment,
      markRead,
      applyRemoteRead: clearCount,
      refresh,
    }),
    [counts, increment, markRead, clearCount, refresh],
  );
}
