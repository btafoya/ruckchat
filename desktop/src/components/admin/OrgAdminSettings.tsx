import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { OrganizationSettings, UpdateOrganizationSettingsRequest } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

interface OrgAdminSettingsProps {
  organizationId: string;
}

export function OrgAdminSettings({ organizationId }: OrgAdminSettingsProps): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [settings, setSettings] = useState<OrganizationSettings | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [form, setForm] = useState<UpdateOrganizationSettingsRequest>({
    max_file_size_bytes: 0,
    storage_quota_bytes: 0,
  });

  const token = session?.token ?? '';

  const load = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const data = await api.orgAdmin.getSettings(token, organizationId);
      setSettings(data);
      setForm({
        max_file_size_bytes: data.max_file_size_bytes,
        storage_quota_bytes: data.storage_quota_bytes,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, organizationId]);

  useEffect(() => {
    void load();
  }, [load]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) return;
    setSaving(true);
    setError(null);
    try {
      await api.orgAdmin.updateSettings(token, organizationId, form);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-xl font-semibold">Organization Settings</h2>

      {error && <div className="rounded bg-danger-bg p-3 text-danger">{error}</div>}

      {isLoading || !settings ? (
        <div className="text-text-muted">Loading...</div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="flex flex-col gap-1">
            <label className="text-sm text-text-muted">Max file size (bytes)</label>
            <input
              type="number"
              value={form.max_file_size_bytes}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  max_file_size_bytes: Number(e.target.value),
                }))
              }
              className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm text-text-muted">Storage quota (bytes)</label>
            <input
              type="number"
              value={form.storage_quota_bytes}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  storage_quota_bytes: Number(e.target.value),
                }))
              }
              className="rounded bg-surface px-3 py-2 text-sm outline-none ring-accent focus:ring"
            />
          </div>

          <button
            type="submit"
            disabled={saving}
            className="rounded bg-accent px-4 py-2 text-sm font-medium text-text-inverse hover:bg-accent-hover disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </form>
      )}
    </div>
  );
}
