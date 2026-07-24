import { useCallback, useState } from 'react';
import type { JSX } from 'react';
import { NavLink } from 'react-router-dom';
import { useSettings } from '../hooks';
import { DEFAULT_API_URL } from '../config';

const THEME_OPTIONS: Array<{ value: 'light' | 'dark' | 'system'; label: string }> = [
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
  { value: 'system', label: 'System' },
];

export function Settings(): JSX.Element {
  const {
    apiUrl,
    notificationsEnabled,
    theme,
    setApiUrl,
    setNotificationsEnabled,
    setTheme,
    reset,
  } = useSettings();
  const [url, setUrl] = useState(apiUrl);
  const [saved, setSaved] = useState(false);

  const handleSave = useCallback(() => {
    setApiUrl(url);
    setSaved(true);
    const timeout = setTimeout(() => setSaved(false), 2000);
    return () => clearTimeout(timeout);
  }, [setApiUrl, url]);

  const handleReset = useCallback(() => {
    reset();
    setUrl(DEFAULT_API_URL);
  }, [reset]);

  return (
    <div className="flex h-full flex-col bg-bg p-6 text-text">
      <header className="mb-6 flex items-center justify-between">
        <h1 className="text-xl font-semibold">Settings</h1>
        <NavLink to="/" className="text-sm text-text-muted hover:text-text">
          Back
        </NavLink>
      </header>

      <div className="max-w-md space-y-6">
        <div>
          <label htmlFor="api-url" className="mb-1 block text-sm font-medium text-text">
            Server URL
          </label>
          <input
            id="api-url"
            type="url"
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            className="w-full rounded-md border border-border bg-surface p-2 text-sm text-text placeholder:text-text-muted focus:border-accent focus:outline-none"
          />
          <p className="mt-1 text-xs text-text-muted">Backend address used for REST and WebSocket.</p>
        </div>

        <div className="flex items-center gap-3">
          <input
            id="notifications"
            type="checkbox"
            checked={notificationsEnabled}
            onChange={(event) => setNotificationsEnabled(event.target.checked)}
            className="h-4 w-4 accent-accent"
          />
          <label htmlFor="notifications" className="text-sm text-text">
            Enable notifications for direct messages and mentions
          </label>
        </div>

        <div>
          <span className="mb-1 block text-sm font-medium text-text">Theme</span>
          <div className="flex gap-2">
            {THEME_OPTIONS.map((option) => (
              <button
                key={option.value}
                type="button"
                onClick={() => setTheme(option.value)}
                className={`rounded-md px-3 py-1.5 text-sm ${
                  theme === option.value
                    ? 'bg-accent text-text-inverse'
                    : 'bg-surface text-text hover:bg-surface-elevated'
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => void handleSave()}
            className="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-text-inverse hover:bg-accent-hover"
          >
            Save
          </button>
          <button
            type="button"
            onClick={handleReset}
            className="rounded-md px-4 py-2 text-sm text-text hover:bg-surface"
          >
            Reset
          </button>
          {saved && <span className="text-sm text-accent">Saved</span>}
        </div>
      </div>
    </div>
  );
}
