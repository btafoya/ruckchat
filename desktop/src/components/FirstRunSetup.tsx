import { useCallback, useMemo, useState, type JSX } from 'react';
import { ApiClient } from '../api/client';
import { useSettingsContext } from '../context';
import type { ThemePreference } from '../hooks/useSettings';

const THEME_OPTIONS: Array<{ value: ThemePreference; label: string }> = [
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
  { value: 'system', label: 'System' },
];

function normalizeUrl(url: string): string {
  const trimmed = url.trim();
  if (!trimmed) {
    return '';
  }
  // Remove trailing slash so path concatenation in ApiClient works correctly.
  return trimmed.replace(/\/$/, '');
}

export function FirstRunSetup(): JSX.Element {
  const {
    apiUrl,
    theme,
    notificationsEnabled,
    setApiUrl,
    setTheme,
    setNotificationsEnabled,
    setServerUrlConfigured,
  } = useSettingsContext();

  const [url, setUrl] = useState(apiUrl);
  const [selectedTheme, setSelectedTheme] = useState<ThemePreference>(theme);
  const [enableNotifications, setEnableNotifications] = useState(notificationsEnabled);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const normalizedUrl = useMemo(() => normalizeUrl(url), [url]);

  const handleSubmit = useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      setError(null);

      if (!normalizedUrl) {
        setError('Please enter a server URL.');
        return;
      }
      if (!/^https?:\/\//i.test(normalizedUrl)) {
        setError('URL must start with http:// or https://.');
        return;
      }

      setIsSubmitting(true);
      try {
        const client = new ApiClient(normalizedUrl);
        // Registration status is an unauthenticated endpoint on every RuckChat server.
        const response = await client.request<{ allow_registration: boolean }>(
          '/auth/registration-status',
        );
        if (typeof response.allow_registration !== 'boolean') {
          throw new Error('Server did not return a valid RuckChat response.');
        }
        setApiUrl(normalizedUrl);
        setTheme(selectedTheme);
        setNotificationsEnabled(enableNotifications);
        setServerUrlConfigured(true);
      } catch (err) {
        setError(
          err instanceof Error
            ? `Could not reach a RuckChat server at this URL: ${err.message}`
            : 'Could not reach a RuckChat server at this URL.',
        );
      } finally {
        setIsSubmitting(false);
      }
    },
    [
      normalizedUrl,
      selectedTheme,
      enableNotifications,
      setApiUrl,
      setTheme,
      setNotificationsEnabled,
      setServerUrlConfigured,
    ],
  );

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-bg p-4 text-text">
      <div className="w-full max-w-md rounded-lg bg-surface p-6 shadow-lg">
        <h1 className="text-xl font-semibold">Welcome to RuckChat</h1>
        <p className="mt-2 text-sm text-text-muted">
          Enter the address of your RuckChat server to get started.
        </p>

        <form className="mt-6 space-y-6" onSubmit={(e) => void handleSubmit(e)}>
          <div>
            <label htmlFor="setup-url" className="mb-1 block text-sm font-medium text-text">
              Server URL
            </label>
            <input
              id="setup-url"
              type="url"
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://ruckchat.example.com"
              required
              className="w-full rounded-md border border-border bg-bg p-2 text-sm text-text placeholder:text-text-muted focus:border-accent focus:outline-none"
            />
            <p className="mt-1 text-xs text-text-muted">
              The address used for REST API and WebSocket connections.
            </p>
          </div>

          <div>
            <span className="mb-1 block text-sm font-medium text-text">Theme</span>
            <div className="flex gap-2">
              {THEME_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => setSelectedTheme(option.value)}
                  className={`rounded-md px-3 py-1.5 text-sm ${
                    selectedTheme === option.value
                      ? 'bg-accent text-text-inverse'
                      : 'bg-bg text-text hover:bg-surface-elevated'
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          <div className="flex items-center gap-3">
            <input
              id="setup-notifications"
              type="checkbox"
              checked={enableNotifications}
              onChange={(event) => setEnableNotifications(event.target.checked)}
              className="h-4 w-4 accent-accent"
            />
            <label htmlFor="setup-notifications" className="text-sm text-text">
              Enable notifications for direct messages and mentions
            </label>
          </div>

          {error && (
            <div role="alert" className="rounded-md bg-danger-bg p-3 text-sm text-danger">
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={isSubmitting || !normalizedUrl}
            className="w-full rounded-md bg-accent px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
          >
            {isSubmitting ? 'Connecting...' : 'Connect'}
          </button>
        </form>
      </div>
    </div>
  );
}
