import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { ServerUser } from '../../api';
import { useSessionContext, useSettingsContext } from '../../context';
import { ConfirmationModal } from './ConfirmationModal';

export function ServerAdminAdmins(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [admins, setAdmins] = useState<ServerUser[]>([]);
  const [candidates, setCandidates] = useState<ServerUser[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [showAdd, setShowAdd] = useState(false);
  const [confirmDemote, setConfirmDemote] = useState<ServerUser | null>(null);
  const [confirmPromote, setConfirmPromote] = useState<ServerUser | null>(null);

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
    } finally {
      setConfirmPromote(null);
    }
  };

  const demote = async (userId: string) => {
    if (!token) return;
    try {
      await api.serverAdmin.demoteUser(token, userId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to demote user');
    } finally {
      setConfirmDemote(null);
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
          className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover"
        >
          {showAdd ? 'Cancel' : 'Add Admin'}
        </button>
      </div>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      {showAdd && (
        <div className="space-y-3 rounded border border-border p-3">
          <div className="text-sm text-text">
            Search active users below and click <strong>Promote</strong> to grant server
            administrator access.
          </div>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by email or display name"
            className="w-full rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
          />
          {filteredCandidates.length === 0 ? (
            <div className="text-sm text-text-muted">No eligible users found.</div>
          ) : (
            <ul className="max-h-64 divide-y divide-border overflow-auto rounded border border-border">
              {filteredCandidates.map((user) => (
                <li key={user.id} className="flex items-center justify-between px-3 py-2">
                  <div>
                    <div className="text-sm font-medium">{user.display_name}</div>
                    <div className="text-xs text-text-muted">{user.email}</div>
                  </div>
                  <button
                    type="button"
                    onClick={() => setConfirmPromote(user)}
                    className="text-xs text-accent hover:text-accent-hover"
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
        <div className="text-text-muted">Loading...</div>
      ) : (
        <ul className="divide-y divide-border">
          {admins.map((admin) => (
            <li key={admin.id} className="flex items-center justify-between py-3">
              <div>
                <div className="font-medium">{admin.display_name}</div>
                <div className="text-sm text-text-muted">{admin.email}</div>
              </div>
              {admins.length > 1 && (
                <button
                  type="button"
                  onClick={() => setConfirmDemote(admin)}
                  className="text-xs text-danger hover:text-danger-hover"
                >
                  Demote
                </button>
              )}
            </li>
          ))}
        </ul>
      )}

      {confirmPromote && (
        <ConfirmationModal
          title="Promote to server admin"
          message={`Are you sure you want to promote ${confirmPromote.display_name} to server administrator?`}
          confirmLabel="Promote"
          onConfirm={() => void promote(confirmPromote.id)}
          onCancel={() => setConfirmPromote(null)}
        />
      )}

      {confirmDemote && (
        <ConfirmationModal
          title="Demote server admin"
          message={`Are you sure you want to remove server administrator privileges from ${confirmDemote.display_name}?`}
          confirmLabel="Demote"
          confirmDanger
          onConfirm={() => void demote(confirmDemote.id)}
          onCancel={() => setConfirmDemote(null)}
        />
      )}
    </div>
  );
}
