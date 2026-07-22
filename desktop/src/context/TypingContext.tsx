import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { TypingState } from '../hooks/useTyping';

export const TypingContext = createContext<TypingState | null>(null);

export function useTypingContext(): TypingState {
  const value = useContext(TypingContext);
  if (!value) {
    throw new Error('useTypingContext must be used within a TypingProvider');
  }
  return value;
}

interface TypingProviderProps {
  value: TypingState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function TypingProvider({ value, children }: TypingProviderProps): JSX.Element {
  return <TypingContext.Provider value={value}>{children}</TypingContext.Provider>;
}
