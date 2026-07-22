import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { WebSocketState } from '../hooks/useWebsocket';

export const RealtimeContext = createContext<WebSocketState | null>(null);

export function useRealtimeContext(): WebSocketState {
  const value = useContext(RealtimeContext);
  if (!value) {
    throw new Error('useRealtimeContext must be used within a RealtimeProvider');
  }
  return value;
}

interface RealtimeProviderProps {
  value: WebSocketState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function RealtimeProvider({ value, children }: RealtimeProviderProps): JSX.Element {
  return <RealtimeContext.Provider value={value}>{children}</RealtimeContext.Provider>;
}
