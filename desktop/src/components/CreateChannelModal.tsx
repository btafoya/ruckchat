import { useMemo, useState } from 'react';
import type { JSX } from 'react';
import { useNavigate } from 'react-router-dom';
import { createApi } from '../api';
import { useChannelContext, useOrgMemberContext, useSessionContext } from '../context';
import { useSettings } from '../hooks';

interface CreateChannelModalProps {
  organizationId: string;
  onClose: () => void;
}

export function CreateChannelModal({ organizationId, onClose }: CreateChannelModalProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const { refresh } = useChannelContext();
  const { members } = useOrgMemberContext();
  const navigate = useNavigate();

  const [name, setName] = useState('');
  const [isPrivate, setIsPrivate] = useState(false);
  const [topic, setTopic] = useState('');
  const [purpose, setPurpose] = useState('');
  const [inviteeIds, setInviteeIds] = useState<string[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggleInvitee = (userId: string) => {
    setInviteeIds((prev) =>
      prev.includes(userId) ? prev.filter((id) => id !== userId) : [...prev, userId],
    );
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!session || isSubmitting) {
      return;
    }
    setIsSubmitting(true);
    setError(null);
    try {
      const channel = await api.organizations.createChannel(session.token, organizationId, {
        name,
        is_private: isPrivate,
      });

      if (topic.trim() || purpose.trim()) {
        await api.channels.update(session.token, channel.id, {
          topic: topic.trim() || null,
          purpose: purpose.trim() || null,
        });
      }

      if (isPrivate) {
        await Promise.all(
          inviteeIds.map((userId) => api.channels.addMember(session.token, channel.id, userId)),
        );
      }

      await refresh();
      onClose();
      navigate(`/org/${organizationId}/channel/${channel.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create channel');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-overlay p-4">
      <div className="flex w-full max-w-md flex-col gap-4 rounded-lg bg-surface p-6 shadow-lg">
        <h2 className="text-lg font-semibold text-text">Create a channel</h2>
        {error && (
          <div role="alert" className="rounded-md bg-danger-bg p-3 text-sm text-danger">
            {error}
          </div>
        )}
        <form className="flex flex-col gap-4" onSubmit={(e) => void handleSubmit(e)}>
          <label className="flex flex-col gap-1 text-sm text-text">
            Name
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              pattern="[a-z0-9][a-z0-9\-]{0,62}[a-z0-9]"
              placeholder="team-updates"
              className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
            />
          </label>

          <label className="flex items-center gap-2 text-sm text-text">
            <input
              type="checkbox"
              checked={isPrivate}
              onChange={(e) => setIsPrivate(e.target.checked)}
            />
            Private channel
          </label>

          <label className="flex flex-col gap-1 text-sm text-text">
            Topic (optional)
            <input
              type="text"
              value={topic}
              onChange={(e) => setTopic(e.target.value)}
              className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
            />
          </label>

          <label className="flex flex-col gap-1 text-sm text-text">
            Purpose (optional)
            <input
              type="text"
              value={purpose}
              onChange={(e) => setPurpose(e.target.value)}
              className="rounded-md border border-border bg-bg px-3 py-2 text-text focus:border-accent focus:outline-none"
            />
          </label>

          {isPrivate && members.length > 0 && (
            <fieldset className="flex flex-col gap-1">
              <legend className="text-sm text-text">Invite members (optional)</legend>
              <div className="flex max-h-40 flex-col gap-1 overflow-y-auto rounded-md border border-border p-2">
                {members
                  .filter((m) => m.id !== session?.user.id)
                  .map((member) => (
                    <label key={member.id} className="flex items-center gap-2 text-sm text-text">
                      <input
                        type="checkbox"
                        checked={inviteeIds.includes(member.id)}
                        onChange={() => toggleInvitee(member.id)}
                      />
                      {member.display_name || member.email}
                    </label>
                  ))}
              </div>
            </fieldset>
          )}

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-md px-4 py-2 text-sm text-text hover:bg-surface-elevated"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting || !name.trim()}
              className="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
            >
              {isSubmitting ? 'Creating...' : 'Create channel'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
