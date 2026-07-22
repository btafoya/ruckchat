import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi } from '../api';
import type { DirectMessageConversation } from '../api';

export interface DirectMessagesState {
  conversations: DirectMessageConversation[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export interface UseDirectMessagesOptions {
  apiUrl?: string;
}

export function useDirectMessages(
  token: string | undefined,
  organizationId: string | undefined,
  options: UseDirectMessagesOptions = {},
): DirectMessagesState {
  const [conversations, setConversations] = useState<DirectMessageConversation[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useMemo(() => createApi(options.apiUrl), [options.apiUrl]);

  const refresh = useCallback(async () => {
    if (!token || !organizationId) {
      setConversations([]);
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.directMessages.list(token, organizationId);
      setConversations(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load direct messages');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    conversations,
    isLoading,
    error,
    refresh,
  };
}
