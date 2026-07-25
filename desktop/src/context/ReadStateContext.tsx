import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { ReadState } from '../hooks/useReadState';

export const ReadStateContext = createContext<ReadState | null>(null);

export function useReadStateContext(): ReadState {
  const value = useContext(ReadStateContext);
  if (!value) {
    throw new Error('useReadStateContext must be used within a ReadStateProvider');
  }
  return value;
}

interface ReadStateProviderProps {
  value: ReadState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function ReadStateProvider({ value, children }: ReadStateProviderProps): JSX.Element {
  return <ReadStateContext.Provider value={value}>{children}</ReadStateContext.Provider>;
}
