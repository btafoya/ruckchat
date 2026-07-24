import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi } from '../api';
import type { User } from '../api';

export interface OrgMembersState {
  members: User[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export interface UseOrgMembersOptions {
  apiUrl?: string;
}

export function useOrgMembers(
  token: string | undefined,
  organizationId: string | undefined,
  options: UseOrgMembersOptions = {},
): OrgMembersState {
  const [members, setMembers] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useMemo(() => createApi(options.apiUrl), [options.apiUrl]);

  const refresh = useCallback(async () => {
    if (!token || !organizationId) {
      setMembers([]);
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.organizations.searchMembers(token, organizationId, '');
      setMembers(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load organization members');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    members,
    isLoading,
    error,
    refresh,
  };
}
