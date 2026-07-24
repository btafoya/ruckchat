import { useEffect, useMemo, useState } from 'react';
import type { JSX } from 'react';
import { createApi } from '../api';
import type { Channel, ChannelMembership } from '../api';
import { useChannelContext, useOrgMemberContext, useSessionContext } from '../context';
import { useSettings } from '../hooks';

interface ChannelSettingsModalProps {
  channel: Channel;
  onClose: () => void;
}

export function ChannelSettingsModal({ channel, onClose }: ChannelSettingsModalProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const { refresh } = useChannelContext();
  const { members: orgMembers } = useOrgMemberContext();

  const [topic, setTopic] = useState(channel.topic ?? '');
  const [purpose, setPurpose] = useState(channel.purpose ?? '');
  const [channelMembers, setChannelMembers] = useState<ChannelMembership[]>([]);
  const [addUserId, setAddUserId] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isArchived = channel.archived_at !== null;

  useEffect(() => {
    if (!channel.is_private || !session) {
      return;
    }
    api.channels
      .listMembers(session.token, channel.id)
      .then(setChannelMembers)
      .catch(() => setChannelMembers([]));
  }, [api, channel.id, channel.is_private, session]);

  const memberLabel = (userId: string) =>
    orgMembers.find((m) => m.id === userId)?.display_name ?? userId;

  const invitableMembers = orgMembers.filter(
    (m) => m.id !== session?.user.id && !channelMembers.some((cm) => cm.user_id === m.id),
  );

  const runAction = async (action: () => Promise<void>) => {
    if (!session || isSaving) {
      return;
    }
    setIsSaving(true);
    setError(null);
    try {
      await action();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Action failed');
    } finally {
      setIsSaving(false);
    }
  };

  const handleSave = () =>
    runAction(async () => {
      if (!session) return;
      await api.channels.update(session.token, channel.id, {
        topic: topic.trim() || null,
        purpose: purpose.trim() || null,
      });
      await refresh();
      onClose();
    });

  const handleArchiveToggle = () =>
    runAction(async () => {
      if (!session) return;
      if (isArchived) {
        await api.channels.unarchive(session.token, channel.id);
      } else {
        await api.channels.archive(session.token, channel.id);
      }
      await refresh();
      onClose();
    });

  const handleAddMember = () =>
    runAction(async () => {
      if (!session || !addUserId) return;
      await api.channels.addMember(session.token, channel.id, addUserId);
      const updated = await api.channels.listMembers(session.token, channel.id);
      setChannelMembers(updated);
      setAddUserId('');
    });

  const handleRemoveMember = (userId: string) =>
    runAction(async () => {
      if (!session) return;
      await api.channels.removeMember(session.token, channel.id, userId);
      const updated = await api.channels.listMembers(session.token, channel.id);
      setChannelMembers(updated);
    });

  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-overlay p-4">
      <div className="flex w-full max-w-md flex-col gap-4 rounded-lg bg-surface p-6 shadow-lg">
        <h2 className="text-lg font-semibold text-text"># {channel.name}</h2>
        {error && (
          <div role="alert" className="rounded-md bg-danger-bg p-3 text-sm text-danger">
            {error}
          </div>
        )}

        <label className="flex flex-col gap-1 text-sm text-text">
          Topic
          <input
            type="text"
            value={topic}
            onChange={(e) => setTopic(e.target.value)}
            className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
          />
        </label>

        <label className="flex flex-col gap-1 text-sm text-text">
          Purpose
          <input
            type="text"
            value={purpose}
            onChange={(e) => setPurpose(e.target.value)}
            className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
          />
        </label>

        {channel.is_private && (
          <fieldset className="flex flex-col gap-2">
            <legend className="text-sm text-text">Members</legend>
            <ul className="flex max-h-32 flex-col gap-1 overflow-y-auto rounded-md border border-border p-2">
              {channelMembers.map((m) => (
                <li key={m.user_id} className="flex items-center justify-between text-sm text-text">
                  {memberLabel(m.user_id)}
                  {m.user_id !== session?.user.id && (
                    <button
                      type="button"
                      onClick={() => void handleRemoveMember(m.user_id)}
                      className="text-xs text-text-muted hover:text-danger"
                    >
                      Remove
                    </button>
                  )}
                </li>
              ))}
            </ul>
            {invitableMembers.length > 0 && (
              <div className="flex gap-2">
                <select
                  value={addUserId}
                  onChange={(e) => setAddUserId(e.target.value)}
                  className="flex-1 rounded-md border border-border bg-bg px-2 py-1 text-sm text-text"
                >
                  <option value="">Invite a member...</option>
                  {invitableMembers.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.display_name || m.email}
                    </option>
                  ))}
                </select>
                <button
                  type="button"
                  disabled={!addUserId || isSaving}
                  onClick={() => void handleAddMember()}
                  className="rounded-md bg-accent px-3 py-1 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
                >
                  Invite
                </button>
              </div>
            )}
          </fieldset>
        )}

        <div className="flex items-center justify-between border-t border-border pt-4">
          <button
            type="button"
            onClick={() => void handleArchiveToggle()}
            disabled={isSaving}
            className="rounded-md px-4 py-2 text-sm text-danger hover:bg-danger-bg disabled:opacity-50"
          >
            {isArchived ? 'Unarchive channel' : 'Archive channel'}
          </button>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-md px-4 py-2 text-sm text-text hover:bg-surface-elevated"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => void handleSave()}
              disabled={isSaving}
              className="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
