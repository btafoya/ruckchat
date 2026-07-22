import { useCallback, useEffect, useMemo, useState } from 'react';

const STORAGE_KEY = 'ruckchat_unread_counts';

export interface UnreadState {
  counts: Record<string, number>;
  increment: (conversationId: string) => void;
  markRead: (conversationId: string) => void;
}

function loadCounts(): Record<string, number> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as unknown;
    if (typeof parsed === 'object' && parsed !== null) {
      return parsed as Record<string, number>;
    }
  } catch {
    // ignore corrupted storage
  }
  return {};
}

function saveCounts(counts: Record<string, number>): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(counts));
  } catch {
    // ignore storage failures
  }
}

export function useUnread(activeConversationId: string | undefined): UnreadState {
  const [counts, setCounts] = useState<Record<string, number>>(loadCounts);

  const increment = useCallback((conversationId: string) => {
    if (conversationId === activeConversationId) {
      return;
    }
    setCounts((prev) => {
      const next = { ...prev, [conversationId]: (prev[conversationId] ?? 0) + 1 };
      saveCounts(next);
      return next;
    });
  }, [activeConversationId]);

  const markRead = useCallback((conversationId: string) => {
    setCounts((prev) => {
      if (!prev[conversationId]) {
        return prev;
      }
      const next = { ...prev };
      delete next[conversationId];
      saveCounts(next);
      return next;
    });
  }, []);

  useEffect(() => {
    if (activeConversationId) {
      markRead(activeConversationId);
    }
  }, [activeConversationId, markRead]);

  return useMemo(
    () => ({
      counts,
      increment,
      markRead,
    }),
    [counts, increment, markRead],
  );
}
