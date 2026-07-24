import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { AuditLogEntry } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

export function ServerAdminAuditLog(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [action, setAction] = useState('');

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.serverAdmin.getAuditLog(token, {
        action: action || undefined,
        limit: 100,
      });
      setEntries(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load audit log');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, action]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Audit Log</h2>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      <div className="flex items-center gap-2">
        <input
          type="text"
          value={action}
          onChange={(e) => setAction(e.target.value)}
          placeholder="Filter by action"
          className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
        />
        <button
          type="button"
          onClick={() => void refresh()}
          className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600"
        >
          Refresh
        </button>
      </div>

      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="border-b border-gray-700 text-gray-400">
            <tr>
              <th className="py-2">Time</th>
              <th className="py-2">Action</th>
              <th className="py-2">Actor</th>
              <th className="py-2">Resource</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {entries.map((entry) => (
              <tr key={entry.id}>
                <td className="py-2 text-gray-400">{entry.occurred_at}</td>
                <td className="py-2">{entry.action}</td>
                <td className="py-2 text-gray-400">{entry.actor_id}</td>
                <td className="py-2 text-gray-400">
                  {entry.resource_type}
                  {entry.resource_id ? ` / ${entry.resource_id}` : ''}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
