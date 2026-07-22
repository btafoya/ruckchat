import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { PresenceState } from '../hooks/usePresence';

export const PresenceContext = createContext<PresenceState | null>(null);

export function usePresenceContext(): PresenceState {
  const value = useContext(PresenceContext);
  if (!value) {
    throw new Error('usePresenceContext must be used within a PresenceProvider');
  }
  return value;
}

interface PresenceProviderProps {
  value: PresenceState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function PresenceProvider({ value, children }: PresenceProviderProps): JSX.Element {
  return <PresenceContext.Provider value={value}>{children}</PresenceContext.Provider>;
}
