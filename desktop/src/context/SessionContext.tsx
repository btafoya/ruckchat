import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { SessionState } from '../hooks/useSession';

export const SessionContext = createContext<SessionState | null>(null);

export function useSessionContext(): SessionState {
  const value = useContext(SessionContext);
  if (!value) {
    throw new Error('useSessionContext must be used within a SessionProvider');
  }
  return value;
}

interface SessionProviderProps {
  value: SessionState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function SessionProvider({ value, children }: SessionProviderProps): JSX.Element {
  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}
