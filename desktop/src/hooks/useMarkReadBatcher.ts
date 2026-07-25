import { useCallback, useEffect, useMemo, useRef } from 'react';
import { createApi } from '../api';

const BATCH_DELAY_MS = 400;

/**
 * Batches per-message read-state calls triggered by scroll-into-view
 * tracking, so a fast scroll through many unread messages fires one
 * network request instead of one per message.
 */
export function useMarkReadBatcher(
  apiUrl: string | undefined,
  token: string | undefined,
  conversationType: 'channel' | 'direct_message' | undefined,
  conversationId: string | undefined,
  markRead: (messageId: string) => void,
): (messageId: string) => void {
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const pendingRef = useRef<Set<string>>(new Set());
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flush = useCallback(() => {
    timerRef.current = null;
    const ids = Array.from(pendingRef.current);
    pendingRef.current = new Set();
    if (!token || !conversationType || !conversationId || ids.length === 0) {
      return;
    }
    const call =
      conversationType === 'channel' ? api.channels.markRead : api.directMessages.markRead;
    void call(token, conversationId, ids).catch((err) => {
      console.warn('Failed to mark messages read', err);
    });
  }, [api, token, conversationType, conversationId]);

  useEffect(
    () => () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    },
    [],
  );

  return useCallback(
    (messageId: string) => {
      pendingRef.current.add(messageId);
      markRead(messageId);
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      timerRef.current = setTimeout(flush, BATCH_DELAY_MS);
    },
    [flush, markRead],
  );
}
