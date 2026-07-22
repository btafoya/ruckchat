import { useCallback, useState } from 'react';
import type { JSX } from 'react';
import { NavLink } from 'react-router-dom';
import { useSettings } from '../hooks';
import { DEFAULT_API_URL } from '../config';

export function Settings(): JSX.Element {
  const { apiUrl, notificationsEnabled, setApiUrl, setNotificationsEnabled, reset } = useSettings();
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
    <div className="flex h-full flex-col bg-gray-900 p-6 text-white">
      <header className="mb-6 flex items-center justify-between">
        <h1 className="text-xl font-semibold">Settings</h1>
        <NavLink to="/" className="text-sm text-gray-300 hover:text-white">
          Back
        </NavLink>
      </header>

      <div className="max-w-md space-y-6">
        <div>
          <label htmlFor="api-url" className="mb-1 block text-sm font-medium text-gray-300">
            Server URL
          </label>
          <input
            id="api-url"
            type="url"
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            className="w-full rounded-md border border-gray-600 bg-gray-800 p-2 text-sm text-white placeholder-gray-500 focus:border-green-500 focus:outline-none"
          />
          <p className="mt-1 text-xs text-gray-400">Backend address used for REST and WebSocket.</p>
        </div>

        <div className="flex items-center gap-3">
          <input
            id="notifications"
            type="checkbox"
            checked={notificationsEnabled}
            onChange={(event) => setNotificationsEnabled(event.target.checked)}
            className="h-4 w-4 accent-green-600"
          />
          <label htmlFor="notifications" className="text-sm text-gray-300">
            Enable notifications for direct messages and mentions
          </label>
        </div>

        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => void handleSave()}
            className="rounded-md bg-green-600 px-4 py-2 text-sm font-semibold text-white hover:bg-green-500"
          >
            Save
          </button>
          <button
            type="button"
            onClick={handleReset}
            className="rounded-md px-4 py-2 text-sm text-gray-300 hover:bg-gray-800"
          >
            Reset
          </button>
          {saved && <span className="text-sm text-green-400">Saved</span>}
        </div>
      </div>
    </div>
  );
}
