import { useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { ServerUser } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

interface EditUserModalProps {
  /** The user to edit, or null to create a new user. */
  user: ServerUser | null;
  onClose: () => void;
  onSaved: () => void;
}

export function EditUserModal({ user, onClose, onSaved }: EditUserModalProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const isCreating = user === null;

  const [displayName, setDisplayName] = useState(user?.display_name ?? '');
  const [email, setEmail] = useState(user?.email ?? '');
  const [avatarUrl, setAvatarUrl] = useState(user?.avatar_url ?? '');
  const [password, setPassword] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const token = session?.token ?? '';

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || isSubmitting) return;
    setIsSubmitting(true);
    setError(null);
    try {
      if (isCreating) {
        const result = await api.serverAdmin.createUser(token, {
          email,
          display_name: displayName,
          password: password || null,
        });
        setNotice(`User created. Initial password: ${result.password}`);
      } else {
        await api.serverAdmin.updateUser(token, user.id, {
          display_name: displayName,
          email,
          avatar_url: avatarUrl || null,
        });
      }
      onSaved();
      if (isCreating) {
        setDisplayName('');
        setEmail('');
        setPassword('');
      } else {
        onClose();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save user');
    } finally {
      setIsSubmitting(false);
    }
  };

  const runAction = async (label: string, fn: () => Promise<unknown>) => {
    if (!token) return;
    setError(null);
    try {
      await fn();
      onSaved();
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${label}`);
    }
  };

  const handleResetPassword = () =>
    runAction('reset password', async () => {
      if (!user) return;
      const result = await api.serverAdmin.resetPassword(token, user.id);
      setNotice(`Temporary password: ${result.password}`);
    });

  const handleTogglePromotion = () =>
    runAction(user?.is_server_admin ? 'demote' : 'promote', () =>
      user?.is_server_admin
        ? api.serverAdmin.demoteUser(token, user.id)
        : api.serverAdmin.promoteUser(token, user!.id),
    );

  const handleToggleActivation = () =>
    runAction(user?.deactivated_at ? 'reactivate' : 'deactivate', () =>
      user?.deactivated_at
        ? api.serverAdmin.reactivateUser(token, user.id)
        : api.serverAdmin.deactivateUser(token, user!.id),
    );

  const handleDelete = async () => {
    if (!user || !token) return;
    if (!window.confirm(`Permanently delete ${user.display_name}? This cannot be undone.`)) {
      return;
    }
    setError(null);
    try {
      await api.serverAdmin.deleteUser(token, user.id);
      onSaved();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete user');
    }
  };

  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-overlay p-4">
      <div className="flex w-full max-w-md flex-col gap-4 rounded-lg bg-surface p-6 shadow-lg">
        <h2 className="text-lg font-semibold text-text">
          {isCreating ? 'Create user' : `Edit ${user.display_name}`}
        </h2>

        {error && (
          <div role="alert" className="rounded-md bg-danger-bg p-3 text-sm text-danger">
            {error}
          </div>
        )}
        {notice && (
          <div role="status" className="rounded-md bg-accent/10 p-3 text-sm text-accent">
            {notice}
          </div>
        )}

        <form className="flex flex-col gap-4" onSubmit={(e) => void handleSubmit(e)}>
          <label className="flex flex-col gap-1 text-sm text-text">
            Display name
            <input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              required
              className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
            />
          </label>

          <label className="flex flex-col gap-1 text-sm text-text">
            Email
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
            />
          </label>

          {isCreating ? (
            <label className="flex flex-col gap-1 text-sm text-text">
              Password (optional)
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Leave blank to generate"
                className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
              />
            </label>
          ) : (
            <label className="flex flex-col gap-1 text-sm text-text">
              Avatar URL
              <input
                type="url"
                value={avatarUrl}
                onChange={(e) => setAvatarUrl(e.target.value)}
                className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
              />
            </label>
          )}

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-md px-4 py-2 text-sm text-text hover:bg-surface-elevated"
            >
              Close
            </button>
            <button
              type="submit"
              disabled={isSubmitting || !displayName.trim() || !email.trim()}
              className="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
            >
              {isSubmitting ? 'Saving...' : isCreating ? 'Create user' : 'Save changes'}
            </button>
          </div>
        </form>

        {!isCreating && (
          <div className="flex flex-col gap-2 rounded-md border border-border p-3">
            <span className="text-sm font-medium text-text">Account</span>
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => void handleResetPassword()}
                className="rounded-md px-3 py-1.5 text-xs text-info hover:bg-surface-elevated"
              >
                Reset password
              </button>
              <button
                type="button"
                onClick={() => void handleTogglePromotion()}
                className="rounded-md px-3 py-1.5 text-xs text-accent hover:bg-surface-elevated"
              >
                {user.is_server_admin ? 'Demote from server admin' : 'Promote to server admin'}
              </button>
              <button
                type="button"
                onClick={() => void handleToggleActivation()}
                className="rounded-md px-3 py-1.5 text-xs text-warning hover:bg-surface-elevated"
              >
                {user.deactivated_at ? 'Reactivate account' : 'Deactivate account'}
              </button>
            </div>
          </div>
        )}

        {!isCreating && (
          <div className="flex flex-col gap-2 rounded-md border border-danger p-3">
            <span className="text-sm font-medium text-danger">Danger zone</span>
            <button
              type="button"
              onClick={() => void handleDelete()}
              className="self-start rounded-md bg-danger px-3 py-1.5 text-xs font-semibold text-text-inverse hover:bg-danger-hover"
            >
              Delete user permanently
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
