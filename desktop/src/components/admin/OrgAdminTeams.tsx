import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { CreateTeamRequest, Team, UpdateTeamRequest } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

interface OrgAdminTeamsProps {
  organizationId: string;
}

export function OrgAdminTeams({ organizationId }: OrgAdminTeamsProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [teams, setTeams] = useState<Team[]>([]);
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
      const items = await api.orgAdmin.listTeams(token, organizationId);
      setTeams(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load teams');
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
    const request: CreateTeamRequest = {
      name: newName,
      description: newDescription || null,
    };
    try {
      await api.orgAdmin.createTeam(token, organizationId, request);
      setNewName('');
      setNewDescription('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create team');
    }
  };

  const handleUpdate = async (teamId: string, name: string, description: string | null) => {
    if (!token) return;
    const request: UpdateTeamRequest = { name, description };
    try {
      await api.orgAdmin.updateTeam(token, organizationId, teamId, request);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update team');
    }
  };

  const handleDelete = async (teamId: string) => {
    if (!token) return;
    if (!window.confirm('Delete this team?')) return;
    try {
      await api.orgAdmin.deleteTeam(token, organizationId, teamId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete team');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Teams</h2>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="Team name"
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
            {teams.map((team) => (
              <TeamRow
                key={team.id}
                team={team}
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

interface TeamRowProps {
  team: Team;
  onUpdate: (id: string, name: string, description: string | null) => void;
  onDelete: (id: string) => void;
}

function TeamRow({ team, onUpdate, onDelete }: TeamRowProps): JSX.Element {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(team.name);
  const [description, setDescription] = useState(team.description ?? '');

  const save = () => {
    onUpdate(team.id, name, description || null);
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
          team.name
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
          team.description ?? '-'
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
            onClick={() => onDelete(team.id)}
            className="text-xs text-red-400 hover:text-red-300"
          >
            Delete
          </button>
        </div>
      </td>
    </tr>
  );
}
