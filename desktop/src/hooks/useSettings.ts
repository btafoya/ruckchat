import { useCallback, useEffect, useMemo, useState } from 'react';
import { DEFAULT_API_URL } from '../config';

const SETTINGS_KEY = 'ruckchat_settings';

export type ThemePreference = 'light' | 'dark' | 'system';

export interface Settings {
  apiUrl: string;
  notificationsEnabled: boolean;
  theme: ThemePreference;
}

export interface SettingsState extends Settings {
  isLoading: boolean;
  resolvedTheme: 'light' | 'dark';
  setApiUrl: (url: string) => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  setTheme: (theme: ThemePreference) => void;
  reset: () => void;
}

function resolveTheme(theme: ThemePreference): 'light' | 'dark' {
  if (theme !== 'system') {
    return theme;
  }
  if (typeof window === 'undefined' || !window.matchMedia) {
    return 'dark';
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) {
      return { apiUrl: DEFAULT_API_URL, notificationsEnabled: true, theme: 'system' };
    }
    const parsed = JSON.parse(raw) as unknown;
    if (typeof parsed === 'object' && parsed !== null) {
      const settings = parsed as Partial<Settings>;
      const theme: ThemePreference =
        settings.theme === 'light' || settings.theme === 'dark' || settings.theme === 'system'
          ? settings.theme
          : 'system';
      return {
        apiUrl: typeof settings.apiUrl === 'string' && settings.apiUrl.trim() ? settings.apiUrl : DEFAULT_API_URL,
        notificationsEnabled: typeof settings.notificationsEnabled === 'boolean' ? settings.notificationsEnabled : true,
        theme,
      };
    }
  } catch {
    // ignore corrupted storage
  }
  return { apiUrl: DEFAULT_API_URL, notificationsEnabled: true, theme: 'system' };
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
  const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>(() => resolveTheme(settings.theme));

  useEffect(() => {
    setIsLoading(false);
  }, []);

  useEffect(() => {
    setResolvedTheme(resolveTheme(settings.theme));
    if (typeof window === 'undefined' || !window.matchMedia) {
      return;
    }
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => setResolvedTheme(resolveTheme(settings.theme));
    media.addEventListener('change', onChange);
    return () => media.removeEventListener('change', onChange);
  }, [settings.theme]);

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

  const setTheme = useCallback((theme: ThemePreference) => {
    setSettings((prev) => {
      const next = { ...prev, theme };
      saveSettings(next);
      return next;
    });
  }, []);

  const reset = useCallback(() => {
    const next = { apiUrl: DEFAULT_API_URL, notificationsEnabled: true, theme: 'system' as const };
    setSettings(next);
    saveSettings(next);
  }, []);

  return useMemo(
    () => ({
      ...settings,
      isLoading,
      resolvedTheme,
      setApiUrl,
      setNotificationsEnabled,
      setTheme,
      reset,
    }),
    [settings, isLoading, resolvedTheme, setApiUrl, setNotificationsEnabled, setTheme, reset],
  );
}
