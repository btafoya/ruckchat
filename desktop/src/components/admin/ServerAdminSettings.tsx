import { useCallback, useEffect, useMemo, useState, type JSX } from 'react';
import { createApi } from '../../api';
import type { ServerSettings, UpdateServerSettingsRequest } from '../../api';
import { useSessionContext } from '../../context';
import { useSettings } from '../../hooks';

export function ServerAdminSettings(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [settings, setSettings] = useState<ServerSettings | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [form, setForm] = useState<UpdateServerSettingsRequest>({
    maintenance_mode_enabled: false,
    default_max_file_size_bytes: 0,
    default_storage_quota_bytes: 0,
    allowed_signup_domains: [],
  });

  const token = session?.token ?? '';

  const load = useCallback(async () => {
    if (!token) return;
    setIsLoading(true);
    setError(null);
    try {
      const data = await api.serverAdmin.getSettings(token);
      setSettings(data);
      setForm({
        maintenance_mode_enabled: data.maintenance_mode_enabled,
        default_max_file_size_bytes: data.default_max_file_size_bytes,
        default_storage_quota_bytes: data.default_storage_quota_bytes,
        allowed_signup_domains: data.allowed_signup_domains,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setIsLoading(false);
    }
  }, [api, token]);

  useEffect(() => {
    void load();
  }, [load]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token) return;
    setSaving(true);
    setError(null);
    try {
      await api.serverAdmin.updateSettings(token, form);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-xl font-semibold">Server Settings</h2>

      {error && <div className="rounded bg-red-900/50 p-3 text-red-200">{error}</div>}

      {isLoading || !settings ? (
        <div className="text-gray-400">Loading...</div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={form.maintenance_mode_enabled}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  maintenance_mode_enabled: e.target.checked,
                }))
              }
            />
            <span>Maintenance mode</span>
          </label>

          <div className="flex flex-col gap-1">
            <label className="text-sm text-gray-400">Default max file size (bytes)</label>
            <input
              type="number"
              value={form.default_max_file_size_bytes}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  default_max_file_size_bytes: Number(e.target.value),
                }))
              }
              className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm text-gray-400">Default storage quota (bytes)</label>
            <input
              type="number"
              value={form.default_storage_quota_bytes}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  default_storage_quota_bytes: Number(e.target.value),
                }))
              }
              className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm text-gray-400">
              Allowed signup domains (one per line)
            </label>
            <textarea
              value={form.allowed_signup_domains.join('\n')}
              onChange={(e) =>
                setForm((prev) => ({
                  ...prev,
                  allowed_signup_domains: e.target.value
                    .split('\n')
                    .map((d) => d.trim())
                    .filter(Boolean),
                }))
              }
              rows={4}
              className="rounded bg-gray-800 px-3 py-2 text-sm outline-none ring-green-500 focus:ring"
            />
          </div>

          <p className="text-xs text-gray-500">
            Values set in ruckchat.yaml override these settings at runtime.
          </p>

          <button
            type="submit"
            disabled={saving}
            className="rounded bg-green-700 px-4 py-2 text-sm font-medium hover:bg-green-600 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </form>
      )}
    </div>
  );
}
