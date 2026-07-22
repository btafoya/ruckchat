import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { DirectMessagesState } from '../hooks/useDirectMessages';

export const DirectMessageContext = createContext<DirectMessagesState | null>(null);

export function useDirectMessageContext(): DirectMessagesState {
  const value = useContext(DirectMessageContext);
  if (!value) {
    throw new Error('useDirectMessageContext must be used within a DirectMessageProvider');
  }
  return value;
}

interface DirectMessageProviderProps {
  value: DirectMessagesState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function DirectMessageProvider({ value, children }: DirectMessageProviderProps): JSX.Element {
  return <DirectMessageContext.Provider value={value}>{children}</DirectMessageContext.Provider>;
}
