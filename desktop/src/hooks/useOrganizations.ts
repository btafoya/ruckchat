import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi } from '../api';
import type { Organization } from '../api';

export interface OrganizationsState {
  organizations: Organization[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export interface UseOrganizationsOptions {
  apiUrl?: string;
}

export function useOrganizations(
  token: string | undefined,
  options: UseOrganizationsOptions = {},
): OrganizationsState {
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useMemo(() => createApi(options.apiUrl), [options.apiUrl]);

  const refresh = useCallback(async () => {
    if (!token) {
      setOrganizations([]);
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.organizations.list(token);
      setOrganizations(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load organizations');
    } finally {
      setIsLoading(false);
    }
  }, [api, token]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    organizations,
    isLoading,
    error,
    refresh,
  };
}
