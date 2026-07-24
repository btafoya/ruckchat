import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { Organization } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

export function ServerAdminOrganizations(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const [newSlug, setNewSlug] = useState('');

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.serverAdmin.listOrganizations(token);
      setOrganizations(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load organizations');
    } finally {
      setIsLoading(false);
    }
  }, [api, token]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !newName || !newSlug) return;
    try {
      await api.serverAdmin.createOrganization(token, {
        name: newName,
        slug: newSlug,
      });
      setNewName('');
      setNewSlug('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create organization');
    }
  };

  const handleRename = async (id: string, name: string) => {
    if (!token) return;
    try {
      await api.serverAdmin.renameOrganization(token, id, { name });
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to rename organization');
    }
  };

  const handleDelete = async (id: string) => {
    if (!token) return;
    if (!window.confirm('Delete this organization? This cannot be undone.')) return;
    try {
      await api.serverAdmin.deleteOrganization(token, id);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete organization');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Organizations</h2>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-gray-400">Name</label>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
            placeholder="Organization name"
          />
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs text-gray-400">Slug</label>
          <input
            type="text"
            value={newSlug}
            onChange={(e) => setNewSlug(e.target.value)}
            className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
            placeholder="unique-slug"
          />
        </div>
        <button
          type="submit"
          className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600 disabled:opacity-50"
          disabled={!newName || !newSlug}
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
              <th className="py-2">Slug</th>
              <th className="py-2">Owner</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {organizations.map((org) => (
              <OrganizationRow
                key={org.id}
                organization={org}
                onRename={handleRename}
                onDelete={handleDelete}
              />
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

interface OrganizationRowProps {
  organization: Organization;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
}

function OrganizationRow({ organization, onRename, onDelete }: OrganizationRowProps): JSX.Element {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(organization.name);

  const save = () => {
    onRename(organization.id, name);
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
            onBlur={save}
            onKeyDown={(e) => {
              if (e.key === 'Enter') save();
            }}
            className="rounded bg-gray-800 px-2 py-1 text-sm outline-none ring-green-500 focus:ring"
            autoFocus
          />
        ) : (
          <span onClick={() => setEditing(true)} className="cursor-pointer hover:text-green-400">
            {organization.name}
          </span>
        )}
      </td>
      <td className="py-2 text-gray-400">{organization.slug}</td>
      <td className="py-2 text-gray-400">{organization.owner_id}</td>
      <td className="py-2">
        <button
          type="button"
          onClick={() => onDelete(organization.id)}
          className="text-xs text-red-400 hover:text-red-300"
        >
          Delete
        </button>
      </td>
    </tr>
  );
}
