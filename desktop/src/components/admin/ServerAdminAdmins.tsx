import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { ServerUser } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

export function ServerAdminAdmins(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [admins, setAdmins] = useState<ServerUser[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.serverAdmin.listServerAdmins(token);
      setAdmins(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load admins');
    } finally {
      setIsLoading(false);
    }
  }, [api, token]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Server Administrators</h2>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : (
        <ul className="divide-y divide-gray-800">
          {admins.map((admin) => (
            <li key={admin.id} className="py-3">
              <div className="font-medium">{admin.display_name}</div>
              <div className="text-sm text-gray-400">{admin.email}</div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
