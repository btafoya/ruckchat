import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type {
  CreatePermissionRequest,
  Permission,
  UpdatePermissionRequest,
} from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

interface OrgAdminPermissionsProps {
  organizationId: string;
}

export function OrgAdminPermissions({
  organizationId,
}: OrgAdminPermissionsProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [newKey, setNewKey] = useState('');
  const [newDescription, setNewDescription] = useState('');

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.orgAdmin.listPermissions(token, organizationId);
      setPermissions(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load permissions');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !newKey) return;
    const request: CreatePermissionRequest = {
      key: newKey,
      description: newDescription || null,
    };
    try {
      await api.orgAdmin.createPermission(token, organizationId, request);
      setNewKey('');
      setNewDescription('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create permission');
    }
  };

  const handleUpdate = async (
    permissionId: string,
    key: string,
    description: string | null,
  ) => {
    if (!token) return;
    const request: UpdatePermissionRequest = { key, description };
    try {
      await api.orgAdmin.updatePermission(token, organizationId, permissionId, request);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update permission');
    }
  };

  const handleDelete = async (permissionId: string) => {
    if (!token) return;
    if (!window.confirm('Delete this permission?')) return;
    try {
      await api.orgAdmin.deletePermission(token, organizationId, permissionId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete permission');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Permissions</h2>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3">
        <input
          type="text"
          value={newKey}
          onChange={(e) => setNewKey(e.target.value)}
          placeholder="Permission key"
          className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
        />
        <input
          type="text"
          value={newDescription}
          onChange={(e) => setNewDescription(e.target.value)}
          placeholder="Description"
          className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
        />
        <button
          type="submit"
          disabled={!newKey}
          className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          Create
        </button>
      </form>

      {isLoading ? (
        <div className="text-text-muted">Loading...</div>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="border-b border-border text-text-muted">
            <tr>
              <th className="py-2">Key</th>
              <th className="py-2">Description</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {permissions.map((permission) => (
              <PermissionRow
                key={permission.id}
                permission={permission}
                onUpdate={handleUpdate}
                onDelete={handleDelete}
              />
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

interface PermissionRowProps {
  permission: Permission;
  onUpdate: (id: string, key: string, description: string | null) => void;
  onDelete: (id: string) => void;
}

function PermissionRow({ permission, onUpdate, onDelete }: PermissionRowProps): JSX.Element {
  const [editing, setEditing] = useState(false);
  const [key, setKey] = useState(permission.key);
  const [description, setDescription] = useState(permission.description ?? '');

  const save = () => {
    onUpdate(permission.id, key, description || null);
    setEditing(false);
  };

  return (
    <tr>
      <td className="py-2">
        {editing ? (
          <input
            type="text"
            value={key}
            onChange={(e) => setKey(e.target.value)}
            className="rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
          />
        ) : (
          permission.key
        )}
      </td>
      <td className="py-2 text-text-muted">
        {editing ? (
          <input
            type="text"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
          />
        ) : (
          permission.description ?? '-'
        )}
      </td>
      <td className="py-2">
        <div className="flex gap-2">
          {editing ? (
            <button
              type="button"
              onClick={save}
              className="text-xs text-accent hover:text-accent-hover"
            >
              Save
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setEditing(true)}
              className="text-xs text-text hover:text-text"
            >
              Edit
            </button>
          )}
          <button
            type="button"
            onClick={() => onDelete(permission.id)}
            className="text-xs text-danger hover:text-danger-hover"
          >
            Delete
          </button>
        </div>
      </td>
    </tr>
  );
}
