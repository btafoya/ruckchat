import { useMemo, useState } from 'react';
import type { JSX } from 'react';
import { useNavigate } from 'react-router-dom';
import { createApi } from '../api';
import { useDirectMessageContext, useOrgMemberContext, useSessionContext, useSettingsContext } from '../context';

interface StartDmModalProps {
  organizationId: string;
  onClose: () => void;
}

export function StartDmModal({ organizationId, onClose }: StartDmModalProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const { refresh } = useDirectMessageContext();
  const { members } = useOrgMemberContext();
  const navigate = useNavigate();

  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toggleSelected = (userId: string) => {
    setSelectedIds((prev) =>
      prev.includes(userId) ? prev.filter((id) => id !== userId) : [...prev, userId],
    );
  };

  const handleSubmit = async () => {
    if (!session || selectedIds.length === 0 || isSubmitting) {
      return;
    }
    setIsSubmitting(true);
    setError(null);
    try {
      const conversation = await api.directMessages.start(session.token, {
        organization_id: organizationId,
        member_ids: selectedIds,
      });
      await refresh();
      onClose();
      navigate(`/org/${organizationId}/dm/${conversation.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start conversation');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-overlay p-4">
      <div className="flex w-full max-w-md flex-col gap-4 rounded-lg bg-surface p-6 shadow-lg">
        <h2 className="text-lg font-semibold text-text">New message</h2>
        {error && (
          <div role="alert" className="rounded-md bg-danger-bg p-3 text-sm text-danger">
            {error}
          </div>
        )}

        <div className="flex max-h-64 flex-col gap-1 overflow-y-auto rounded-md border border-border p-2">
          {members
            .filter((m) => m.id !== session?.user.id)
            .map((member) => (
              <label key={member.id} className="flex items-center gap-2 text-sm text-text">
                <input
                  type="checkbox"
                  checked={selectedIds.includes(member.id)}
                  onChange={() => toggleSelected(member.id)}
                />
                {member.display_name || member.email}
              </label>
            ))}
          {members.length === 0 && (
            <span className="text-sm text-text-muted">No other members in this organization.</span>
          )}
        </div>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-4 py-2 text-sm text-text hover:bg-surface-elevated"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => void handleSubmit()}
            disabled={isSubmitting || selectedIds.length === 0}
            className="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
          >
            {isSubmitting ? 'Starting...' : 'Start conversation'}
          </button>
        </div>
      </div>
    </div>
  );
}
