import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { Channel, CreateTeamRequest, Member, Team, TeamMember, TeamRoom, UpdateTeamRequest } from '../../api';
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
  const [expandedTeamId, setExpandedTeamId] = useState<string | null>(null);

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
      if (expandedTeamId === teamId) setExpandedTeamId(null);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete team');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Teams</h2>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="Team name"
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
          disabled={!newName}
          className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          Create
        </button>
      </form>

      {isLoading ? (
        <div className="text-text-muted">Loading...</div>
      ) : (
        <div className="divide-y divide-border">
          {teams.map((team) => (
            <TeamRow
              key={team.id}
              team={team}
              organizationId={organizationId}
              expanded={expandedTeamId === team.id}
              onToggle={() =>
                setExpandedTeamId((prev) => (prev === team.id ? null : team.id))
              }
              onUpdate={handleUpdate}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface TeamRowProps {
  team: Team;
  organizationId: string;
  expanded: boolean;
  onToggle: () => void;
  onUpdate: (id: string, name: string, description: string | null) => void;
  onDelete: (id: string) => void;
}

function TeamRow({ team, organizationId, expanded, onToggle, onUpdate, onDelete }: TeamRowProps): JSX.Element {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(team.name);
  const [description, setDescription] = useState(team.description ?? '');

  const save = () => {
    onUpdate(team.id, name, description || null);
    setEditing(false);
  };

  return (
    <div className="py-3">
      <div className="flex items-center gap-3">
        <div className="flex-1">
          {editing ? (
            <div className="flex flex-wrap items-end gap-2">
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
              />
              <input
                type="text"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
              />
              <button
                type="button"
                onClick={save}
                className="text-xs text-accent hover:text-accent-hover"
              >
                Save
              </button>
            </div>
          ) : (
            <>
              <div className="font-medium text-text">{team.name}</div>
              <div className="text-xs text-text-muted">{team.description ?? '-'}</div>
            </>
          )}
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={onToggle}
            className="text-xs text-text hover:text-text"
          >
            {expanded ? 'Hide' : 'Manage'}
          </button>
          {!editing && (
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
            onClick={() => onDelete(team.id)}
            className="text-xs text-danger hover:text-danger-hover"
          >
            Delete
          </button>
        </div>
      </div>
      {expanded && (
        <div className="mt-3 grid grid-cols-1 gap-4 rounded border border-border p-3 md:grid-cols-2">
          <TeamMembersPanel organizationId={organizationId} teamId={team.id} />
          <TeamRoomsPanel organizationId={organizationId} teamId={team.id} />
        </div>
      )}
    </div>
  );
}

interface TeamPanelProps {
  organizationId: string;
  teamId: string;
}

function TeamMembersPanel({ organizationId, teamId }: TeamPanelProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [orgMembers, setOrgMembers] = useState<Member[]>([]);
  const [selectedUserId, setSelectedUserId] = useState('');
  const [error, setError] = useState<string | null>(null);

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    try {
      const [teamMembers, allMembers] = await Promise.all([
        api.orgAdmin.listTeamMembers(token, organizationId, teamId),
        api.organizations.listMembers(token, organizationId),
      ]);
      setMembers(teamMembers);
      setOrgMembers(allMembers);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load team members');
    }
  }, [api, token, organizationId, teamId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const availableMembers = orgMembers.filter(
    (m) => !members.some((tm) => tm.user.id === m.user.id),
  );

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !selectedUserId) return;
    try {
      await api.orgAdmin.addTeamMember(token, organizationId, teamId, {
        user_id: selectedUserId,
      });
      setSelectedUserId('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add member');
    }
  };

  const handleRemove = async (userId: string) => {
    if (!token) return;
    try {
      await api.orgAdmin.removeTeamMember(token, organizationId, teamId, userId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove member');
    }
  };

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-semibold text-text">Members</h3>
      {error && <div className="text-xs text-danger">{error}</div>}
      <ul className="divide-y divide-border">
        {members.map((member) => (
          <li key={member.user.id} className="flex items-center justify-between py-1 text-sm">
            <span>
              {member.user.display_name} <span className="text-text-muted">({member.role})</span>
            </span>
            <button
              type="button"
              onClick={() => void handleRemove(member.user.id)}
              className="text-xs text-danger hover:text-danger-hover"
            >
              Remove
            </button>
          </li>
        ))}
        {members.length === 0 && <li className="py-1 text-xs text-text-muted">No members yet.</li>}
      </ul>
      <form onSubmit={handleAdd} className="flex items-end gap-2">
        <select
          value={selectedUserId}
          onChange={(e) => setSelectedUserId(e.target.value)}
          className="flex-1 rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
        >
          <option value="">Select a member</option>
          {availableMembers.map((m) => (
            <option key={m.user.id} value={m.user.id}>
              {m.user.display_name}
            </option>
          ))}
        </select>
        <button
          type="submit"
          disabled={!selectedUserId}
          className="rounded bg-accent px-3 py-1 text-xs font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          Add
        </button>
      </form>
    </div>
  );
}

function TeamRoomsPanel({ organizationId, teamId }: TeamPanelProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [rooms, setRooms] = useState<TeamRoom[]>([]);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [selectedChannelId, setSelectedChannelId] = useState('');
  const [error, setError] = useState<string | null>(null);

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    try {
      const [teamRooms, orgChannels] = await Promise.all([
        api.orgAdmin.listTeamRooms(token, organizationId, teamId),
        api.organizations.listChannels(token, organizationId),
      ]);
      setRooms(teamRooms);
      setChannels(orgChannels);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load team rooms');
    }
  }, [api, token, organizationId, teamId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const channelName = (channelId: string) =>
    channels.find((c) => c.id === channelId)?.name ?? channelId;

  const availableChannels = channels.filter(
    (c) => !rooms.some((r) => r.channel_id === c.id),
  );

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !selectedChannelId) return;
    try {
      await api.orgAdmin.addTeamRoom(token, organizationId, teamId, {
        channel_id: selectedChannelId,
      });
      setSelectedChannelId('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add room');
    }
  };

  const handleRemove = async (channelId: string) => {
    if (!token) return;
    try {
      await api.orgAdmin.removeTeamRoom(token, organizationId, teamId, channelId);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove room');
    }
  };

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-semibold text-text">Rooms</h3>
      {error && <div className="text-xs text-danger">{error}</div>}
      <ul className="divide-y divide-border">
        {rooms.map((room) => (
          <li key={room.channel_id} className="flex items-center justify-between py-1 text-sm">
            <span>#{channelName(room.channel_id)}</span>
            <button
              type="button"
              onClick={() => void handleRemove(room.channel_id)}
              className="text-xs text-danger hover:text-danger-hover"
            >
              Remove
            </button>
          </li>
        ))}
        {rooms.length === 0 && <li className="py-1 text-xs text-text-muted">No rooms linked yet.</li>}
      </ul>
      <form onSubmit={handleAdd} className="flex items-end gap-2">
        <select
          value={selectedChannelId}
          onChange={(e) => setSelectedChannelId(e.target.value)}
          className="flex-1 rounded bg-surface px-2 py-1 text-sm outline-none ring-accent focus:ring"
        >
          <option value="">Select a channel</option>
          {availableChannels.map((c) => (
            <option key={c.id} value={c.id}>
              #{c.name}
            </option>
          ))}
        </select>
        <button
          type="submit"
          disabled={!selectedChannelId}
          className="rounded bg-accent px-3 py-1 text-xs font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          Add
        </button>
      </form>
    </div>
  );
}
