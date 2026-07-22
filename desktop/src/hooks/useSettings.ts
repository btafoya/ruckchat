import { useCallback, useEffect, useMemo, useState } from 'react';
import { DEFAULT_API_URL } from '../config';

const SETTINGS_KEY = 'ruckchat_settings';

export interface Settings {
  apiUrl: string;
  notificationsEnabled: boolean;
}

export interface SettingsState extends Settings {
  isLoading: boolean;
  setApiUrl: (url: string) => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  reset: () => void;
}

function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) {
      return { apiUrl: DEFAULT_API_URL, notificationsEnabled: true };
    }
    const parsed = JSON.parse(raw) as unknown;
    if (typeof parsed === 'object' && parsed !== null) {
      const settings = parsed as Partial<Settings>;
      return {
        apiUrl: typeof settings.apiUrl === 'string' && settings.apiUrl.trim() ? settings.apiUrl : DEFAULT_API_URL,
        notificationsEnabled: typeof settings.notificationsEnabled === 'boolean' ? settings.notificationsEnabled : true,
      };
    }
  } catch {
    // ignore corrupted storage
  }
  return { apiUrl: DEFAULT_API_URL, notificationsEnabled: true };
}

function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // ignore storage failures
  }
}

export function useSettings(): SettingsState {
  const [settings, setSettings] = useState<Settings>(loadSettings);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    setIsLoading(false);
  }, []);

  const setApiUrl = useCallback((url: string) => {
    setSettings((prev) => {
      const next = { ...prev, apiUrl: url.trim() || DEFAULT_API_URL };
      saveSettings(next);
      return next;
    });
  }, []);

  const setNotificationsEnabled = useCallback((enabled: boolean) => {
    setSettings((prev) => {
      const next = { ...prev, notificationsEnabled: enabled };
      saveSettings(next);
      return next;
    });
  }, []);

  const reset = useCallback(() => {
    const next = { apiUrl: DEFAULT_API_URL, notificationsEnabled: true };
    setSettings(next);
    saveSettings(next);
  }, []);

  return useMemo(
    () => ({
      ...settings,
      isLoading,
      setApiUrl,
      setNotificationsEnabled,
      reset,
    }),
    [settings, isLoading, setApiUrl, setNotificationsEnabled, reset],
  );
}
