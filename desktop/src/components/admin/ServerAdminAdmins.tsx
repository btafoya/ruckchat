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
  const [candidates, setCandidates] = useState<ServerUser[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [showAdd, setShowAdd] = useState(false);

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const [adminItems, allUsers] = await Promise.all([
        api.serverAdmin.listServerAdmins(token),
        api.serverAdmin.listUsers(token, { limit: 500 }),
      ]);
      setAdmins(adminItems);
      setCandidates(allUsers.filter((u) => !u.is_server_admin && !u.deactivated_at));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load admins');
    } finally {
      setIsLoading(false);
    }
  }, [api, token]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const promote = async (userId: string) => {
    if (!token) return;
    try {
      await api.serverAdmin.promoteUser(token, userId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to promote user');
    }
  };

  const filteredCandidates = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return candidates;
    return candidates.filter(
      (u) =>
        u.email.toLowerCase().includes(term) ||
        u.display_name.toLowerCase().includes(term),
    );
  }, [candidates, search]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Server Administrators</h2>
        <button
          type="button"
          onClick={() => setShowAdd((prev) => !prev)}
          className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600"
        >
          {showAdd ? 'Cancel' : 'Add Admin'}
        </button>
      </div>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      {showAdd && (
        <div className="space-y-3 rounded border border-gray-700 p-3">
          <div className="text-sm text-gray-300">
            Search active users below and click <strong>Promote</strong> to grant server
            administrator access.
          </div>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by email or display name"
            className="w-full rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
          />
          {filteredCandidates.length === 0 ? (
            <div className="text-sm text-gray-400">No eligible users found.</div>
          ) : (
            <ul className="max-h-64 divide-y divide-gray-800 overflow-auto rounded border border-gray-800">
              {filteredCandidates.map((user) => (
                <li key={user.id} className="flex items-center justify-between px-3 py-2">
                  <div>
                    <div className="text-sm font-medium">{user.display_name}</div>
                    <div className="text-xs text-gray-400">{user.email}</div>
                  </div>
                  <button
                    type="button"
                    onClick={() => void promote(user.id)}
                    className="text-xs text-green-400 hover:text-green-300"
                  >
                    Promote
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

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
