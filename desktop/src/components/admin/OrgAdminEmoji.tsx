import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { CreateEmojiRequest, CustomEmoji } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

interface OrgAdminEmojiProps {
  organizationId: string;
}

export function OrgAdminEmoji({ organizationId }: OrgAdminEmojiProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [emoji, setEmoji] = useState<CustomEmoji[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [shortcode, setShortcode] = useState('');
  const [fileId, setFileId] = useState('');

  const token = session?.token ?? '';

  const refresh = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const items = await api.orgAdmin.listEmoji(token, organizationId);
      setEmoji(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load emoji');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !shortcode || !fileId) return;
    const request: CreateEmojiRequest = { shortcode, file_id: fileId };
    try {
      await api.orgAdmin.createEmoji(token, organizationId, request);
      setShortcode('');
      setFileId('');
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create emoji');
    }
  };

  const handleDelete = async (id: string) => {
    if (!token) return;
    if (!window.confirm('Delete this emoji?')) return;
    try {
      await api.orgAdmin.deleteEmoji(token, organizationId, id);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete emoji');
    }
  };

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold">Custom Emoji</h2>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      <form onSubmit={handleCreate} className="flex flex-wrap items-end gap-3">
        <input
          type="text"
          value={shortcode}
          onChange={(e) => setShortcode(e.target.value)}
          placeholder="shortcode"
          className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
        />
        <input
          type="text"
          value={fileId}
          onChange={(e) => setFileId(e.target.value)}
          placeholder="file id"
          className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
        />
        <button
          type="submit"
          disabled={!shortcode || !fileId}
          className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          Create
        </button>
      </form>

      {isLoading ? (
        <div className="text-text-muted">Loading...</div>
      ) : (
        <ul className="divide-y divide-border">
          {emoji.map((item) => (
            <li key={item.id} className="flex items-center justify-between py-2">
              <span>:{item.shortcode}:</span>
              <button
                type="button"
                onClick={() => handleDelete(item.id)}
                className="text-xs text-danger hover:text-danger-hover"
              >
                Delete
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
