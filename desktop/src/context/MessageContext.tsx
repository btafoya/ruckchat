import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { MessagesState } from '../hooks/useMessages';

export const MessageContext = createContext<MessagesState | null>(null);

export function useMessageContext(): MessagesState {
  const value = useContext(MessageContext);
  if (!value) {
    throw new Error('useMessageContext must be used within a MessageProvider');
  }
  return value;
}

interface MessageProviderProps {
  value: MessagesState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function MessageProvider({ value, children }: MessageProviderProps): JSX.Element {
  return <MessageContext.Provider value={value}>{children}</MessageContext.Provider>;
}
