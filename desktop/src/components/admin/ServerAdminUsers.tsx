import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { ServerUser } from '../../api';
import { useSessionContext, useSettingsContext } from '../../context';
import { EditUserModal } from './EditUserModal';

export function ServerAdminUsers(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [users, setUsers] = useState<ServerUser[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingUserId, setEditingUserId] = useState<'create' | string | null>(null);

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.serverAdmin.listUsers(token, { limit: 100 });
      setUsers(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setIsLoading(false);
    }
  }, [api, token]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Users</h2>
        <button
          type="button"
          onClick={() => setEditingUserId('create')}
          className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover"
        >
          Create User
        </button>
      </div>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      {isLoading ? (
        <div className="text-text-muted">Loading...</div>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="border-b border-border text-text-muted">
            <tr>
              <th className="py-2">Email</th>
              <th className="py-2">Display Name</th>
              <th className="py-2">Server Admin</th>
              <th className="py-2">Status</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {users.map((user) => (
              <tr key={user.id}>
                <td className="py-2">{user.email}</td>
                <td className="py-2">{user.display_name}</td>
                <td className="py-2">{user.is_server_admin ? 'Yes' : 'No'}</td>
                <td className="py-2">
                  {user.deactivated_at ? (
                    <span className="text-danger">Deactivated</span>
                  ) : (
                    <span className="text-accent">Active</span>
                  )}
                </td>
                <td className="py-2">
                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      onClick={() => setEditingUserId(user.id)}
                      className="text-xs text-text hover:text-text"
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      onClick={() =>
                        void (async () => {
                          try {
                            const result = await api.serverAdmin.impersonate(token, {
                              target_user_id: user.id,
                            });
                            window.alert(`Impersonation token: ${result.token}`);
                          } catch (err) {
                            setError(err instanceof Error ? err.message : 'Failed to impersonate');
                          }
                        })()
                      }
                      className="text-xs text-link hover:text-link-hover"
                    >
                      Impersonate
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {editingUserId && (
        <EditUserModal
          user={
            editingUserId === 'create'
              ? null
              : (users.find((u) => u.id === editingUserId) ?? null)
          }
          onClose={() => setEditingUserId(null)}
          onSaved={() => void refresh()}
        />
      )}
    </div>
  );
}
