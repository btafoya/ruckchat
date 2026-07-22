import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { ChannelsState } from '../hooks/useChannels';

export const ChannelContext = createContext<ChannelsState | null>(null);

export function useChannelContext(): ChannelsState {
  const value = useContext(ChannelContext);
  if (!value) {
    throw new Error('useChannelContext must be used within a ChannelProvider');
  }
  return value;
}

interface ChannelProviderProps {
  value: ChannelsState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function ChannelProvider({ value, children }: ChannelProviderProps): JSX.Element {
  return <ChannelContext.Provider value={value}>{children}</ChannelContext.Provider>;
}
