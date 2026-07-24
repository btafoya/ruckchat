import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { CreateRoleRequest, OrganizationRole, UpdateRoleRequest } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

interface OrgAdminRolesProps {
  organizationId: string;
}

export function OrgAdminRoles({ organizationId }: OrgAdminRolesProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [roles, setRoles] = useState<OrganizationRole[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.orgAdmin.listRoles(token, organizationId);
      setRoles(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load roles');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !newName) return;
    const request: CreateRoleRequest = {
      name: newName,
      description: newDescription || null,
    };
    try {
      await api.orgAdmin.createRole(token, organizationId, request);
      setNewName('');
      setNewDescription('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create role');
    }
  };

  const handleUpdate = async (roleId: string, name: string, description: string | null) => {
    if (!token) return;
    const request: UpdateRoleRequest = {
      name,
      description,
    };
    try {
      await api.orgAdmin.updateRole(token, organizationId, roleId, request);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update role');
    }
  };

  const handleDelete = async (roleId: string) => {
    if (!token) return;
    if (!window.confirm('Delete this role?')) return;
    try {
      await api.orgAdmin.deleteRole(token, organizationId, roleId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete role');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Custom Roles</h2>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="Role name"
          className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
        />
        <input
          type="text"
          value={newDescription}
          onChange={(e) => setNewDescription(e.target.value)}
          placeholder="Description"
          className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
        />
        <button
          type="submit"
          disabled={!newName}
          className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600 disabled:opacity-50"
        >
          Create
        </button>
      </form>

      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="border-b border-gray-700 text-gray-400">
            <tr>
              <th className="py-2">Name</th>
              <th className="py-2">Description</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {roles.map((role) => (
              <RoleRow
                key={role.id}
                role={role}
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

interface RoleRowProps {
  role: OrganizationRole;
  onUpdate: (id: string, name: string, description: string | null) => void;
  onDelete: (id: string) => void;
}

function RoleRow({ role, onUpdate, onDelete }: RoleRowProps): JSX.Element {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(role.name);
  const [description, setDescription] = useState(role.description ?? '');

  const save = () => {
    onUpdate(role.id, name, description || null);
    setEditing(false);
  };

  return (
    <tr>
      <td className="py-2">
        {editing ? (
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="rounded bg-gray-800 px-2 py-1 text-sm outline-none ring-green-500 focus:ring"
          />
        ) : (
          role.name
        )}
      </td>
      <td className="py-2 text-gray-400">
        {editing ? (
          <input
            type="text"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="rounded bg-gray-800 px-2 py-1 text-sm outline-none ring-green-500 focus:ring"
          />
        ) : (
          role.description ?? '-'
        )}
      </td>
      <td className="py-2">
        <div className="flex gap-2">
          {editing ? (
            <button
              type="button"
              onClick={save}
              className="text-xs text-green-400 hover:text-green-300"
            >
              Save
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setEditing(true)}
              className="text-xs text-gray-300 hover:text-white"
            >
              Edit
            </button>
          )}
          <button
            type="button"
            onClick={() => onDelete(role.id)}
            className="text-xs text-red-400 hover:text-red-300"
          >
            Delete
          </button>
        </div>
      </td>
    </tr>
  );
}
