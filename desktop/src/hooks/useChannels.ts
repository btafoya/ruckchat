import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi } from '../api';
import type { Channel } from '../api';

export interface ChannelsState {
  channels: Channel[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useChannels(token: string | undefined, organizationId: string | undefined): ChannelsState {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useMemo(() => createApi(), []);

  const refresh = useCallback(async () => {
    if (!token || !organizationId) {
      setChannels([]);
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.organizations.listChannels(token, organizationId);
      setChannels(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load channels');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    channels,
    isLoading,
    error,
    refresh,
  };
}
