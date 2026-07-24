import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { CreateServerUserRequest, ServerUser, UpdateServerUserRequest } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

export function ServerAdminUsers(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [users, setUsers] = useState<ServerUser[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<UpdateServerUserRequest>({});
  const [createForm, setCreateForm] = useState<CreateServerUserRequest>({
    email: '',
    display_name: '',
    password: null,
  });
  const [showCreate, setShowCreate] = useState(false);

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

  const startEdit = (user: ServerUser) => {
    setEditing(user.id);
    setEditForm({
      display_name: user.display_name,
      email: user.email,
      avatar_url: user.avatar_url ?? null,
    });
  };

  const saveEdit = async (userId: string) => {
    if (!token) return;
    try {
      await api.serverAdmin.updateUser(token, userId, editForm);
      setEditing(null);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user');
    }
  };

  const action = async (label: string, fn: () => Promise<unknown>) => {
    try {
      await fn();
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${label}`);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) return;
    try {
      const result = await api.serverAdmin.createUser(token, createForm);
      setCreateForm({ email: '', display_name: '', password: null });
      setShowCreate(false);
      window.alert(`User created. Initial password: ${result.password}`);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create user');
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Users</h2>
        <button
          type="button"
          onClick={() => setShowCreate((prev) => !prev)}
          className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600"
        >
          {showCreate ? 'Cancel' : 'Create User'}
        </button>
      </div>

      {showCreate && (
        <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3 rounded border border-gray-700 p-3">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-400">Email</label>
            <input
              type="email"
              required
              value={createForm.email}
              onChange={(e) => setCreateForm((prev) => ({ ...prev, email: e.target.value }))}
              className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
              placeholder="user@example.com"
            />
          </div>
          <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-400">Display Name</label>
            <input
              type="text"
              required
              value={createForm.display_name}
              onChange={(e) =>
                setCreateForm((prev) => ({ ...prev, display_name: e.target.value }))
              }
              className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
              placeholder="Full name"
            />
          </div>
          <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-400">Password (optional)</label>
            <input
              type="password"
              value={createForm.password ?? ''}
              onChange={(e) =>
                setCreateForm((prev) => ({
                  ...prev,
                  password: e.target.value || null,
                }))
              }
              className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
              placeholder="Leave blank to generate"
            />
          </div>
          <button
            type="submit"
            className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600 disabled:opacity-50"
            disabled={!createForm.email || !createForm.display_name}
          >
            Create
          </button>
        </form>
      )}

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="border-b border-gray-700 text-gray-400">
            <tr>
              <th className="py-2">Email</th>
              <th className="py-2">Display Name</th>
              <th className="py-2">Server Admin</th>
              <th className="py-2">Status</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {users.map((user) => (
              <tr key={user.id}>
                <td className="py-2">
                  {editing === user.id ? (
                    <input
                      type="email"
                      value={editForm.email ?? ''}
                      onChange={(e) =>
                        setEditForm((prev) => ({ ...prev, email: e.target.value }))
                      }
                      className="w-full rounded bg-gray-800 px-2 py-1 text-sm outline-none ring-green-500 focus:ring"
                    />
                  ) : (
                    user.email
                  )}
                </td>
                <td className="py-2">
                  {editing === user.id ? (
                    <input
                      type="text"
                      value={editForm.display_name ?? ''}
                      onChange={(e) =>
                        setEditForm((prev) => ({ ...prev, display_name: e.target.value }))
                      }
                      className="w-full rounded bg-gray-800 px-2 py-1 text-sm outline-none ring-green-500 focus:ring"
                    />
                  ) : (
                    user.display_name
                  )}
                </td>
                <td className="py-2">{user.is_server_admin ? 'Yes' : 'No'}</td>
                <td className="py-2">
                  {user.deactivated_at ? (
                    <span className="text-red-400">Deactivated</span>
                  ) : (
                    <span className="text-green-400">Active</span>
                  )}
                </td>
                <td className="py-2">
                  <div className="flex flex-wrap gap-2">
                    {editing === user.id ? (
                      <button
                        type="button"
                        onClick={() => saveEdit(user.id)}
                        className="text-xs text-green-400 hover:text-green-300"
                      >
                        Save
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() => startEdit(user)}
                        className="text-xs text-gray-300 hover:text-white"
                      >
                        Edit
                      </button>
                    )}
                    {user.is_server_admin ? (
                      <button
                        type="button"
                        onClick={() =>
                          action('demote', () => api.serverAdmin.demoteUser(token, user.id))
                        }
                        className="text-xs text-yellow-400 hover:text-yellow-300"
                      >
                        Demote
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() =>
                          action('promote', () => api.serverAdmin.promoteUser(token, user.id))
                        }
                        className="text-xs text-green-400 hover:text-green-300"
                      >
                        Promote
                      </button>
                    )}
                    {user.deactivated_at ? (
                      <button
                        type="button"
                        onClick={() =>
                          action('reactivate', () =>
                            api.serverAdmin.reactivateUser(token, user.id),
                          )
                        }
                        className="text-xs text-green-400 hover:text-green-300"
                      >
                        Reactivate
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() =>
                          action('deactivate', () =>
                            api.serverAdmin.deactivateUser(token, user.id),
                          )
                        }
                        className="text-xs text-red-400 hover:text-red-300"
                      >
                        Deactivate
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() =>
                        action('reset password', async () => {
                          const result = await api.serverAdmin.resetPassword(token, user.id);
                          window.alert(`Temporary password: ${result.password}`);
                        })
                      }
                      className="text-xs text-blue-400 hover:text-blue-300"
                    >
                      Reset Password
                    </button>
                    <button
                      type="button"
                      onClick={() =>
                        action('impersonate', async () => {
                          const result = await api.serverAdmin.impersonate(token, {
                            target_user_id: user.id,
                          });
                          window.alert(`Impersonation token: ${result.token}`);
                        })
                      }
                      className="text-xs text-purple-400 hover:text-purple-300"
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
    </div>
  );
}
