import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { SettingsState } from '../hooks/useSettings';

export const SettingsContext = createContext<SettingsState | null>(null);

export function useSettingsContext(): SettingsState {
  const value = useContext(SettingsContext);
  if (!value) {
    throw new Error('useSettingsContext must be used within a SettingsProvider');
  }
  return value;
}

interface SettingsProviderProps {
  value: SettingsState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function SettingsProvider({ value, children }: SettingsProviderProps): JSX.Element {
  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}
