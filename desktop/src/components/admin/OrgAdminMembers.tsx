import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { Member, Role } from '../../api';
import { useSessionContext, useSettingsContext } from '../../context';

interface OrgAdminMembersProps {
  organizationId: string;
}

const ROLE_OPTIONS: Role[] = ['owner', 'admin', 'member'];

export function OrgAdminMembers({ organizationId }: OrgAdminMembersProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [members, setMembers] = useState<Member[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState<Role>('member');

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.organizations.listMembers(token, organizationId);
      setMembers(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load members');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleInvite = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !inviteEmail) return;
    try {
      await api.organizations.inviteMember(token, organizationId, {
        email: inviteEmail,
        role: inviteRole,
      });
      setInviteEmail('');
      setInviteRole('member');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to invite member');
    }
  };

  const handleChangeRole = async (userId: string, role: Role) => {
    if (!token) return;
    try {
      await api.organizations.changeRole(token, organizationId, { user_id: userId, role });
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change role');
    }
  };

  const handleRemove = async (userId: string) => {
    if (!token) return;
    if (!window.confirm('Remove this member from the organization?')) return;
    try {
      await api.organizations.removeMember(token, organizationId, userId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove member');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Members</h2>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      <form onSubmit={handleInvite} className="flex flex-wrap items-end gap-3">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-text-muted">Email</label>
          <input
            type="email"
            required
            value={inviteEmail}
            onChange={(e) => setInviteEmail(e.target.value)}
            placeholder="user@example.com"
            className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
          />
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs text-text-muted">Role</label>
          <select
            value={inviteRole}
            onChange={(e) => setInviteRole(e.target.value as Role)}
            className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
          >
            {ROLE_OPTIONS.map((role) => (
              <option key={role} value={role}>
                {role}
              </option>
            ))}
          </select>
        </div>
        <button
          type="submit"
          disabled={!inviteEmail}
          className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          Invite
        </button>
      </form>

      {isLoading ? (
        <div className="text-text-muted">Loading...</div>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="border-b border-border text-text-muted">
            <tr>
              <th className="py-2">Name</th>
              <th className="py-2">Email</th>
              <th className="py-2">Role</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {members.map((member) => (
              <tr key={member.user.id}>
                <td className="py-2">{member.user.display_name}</td>
                <td className="py-2 text-text-muted">{member.user.email}</td>
                <td className="py-2">
                  <select
                    value={member.role}
                    onChange={(e) => void handleChangeRole(member.user.id, e.target.value as Role)}
                    className="rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
                  >
                    {ROLE_OPTIONS.map((role) => (
                      <option key={role} value={role}>
                        {role}
                      </option>
                    ))}
                  </select>
                </td>
                <td className="py-2">
                  <button
                    type="button"
                    onClick={() => void handleRemove(member.user.id)}
                    className="text-xs text-danger hover:text-danger-hover"
                  >
                    Remove
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
