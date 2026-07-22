import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { OrganizationsState } from '../hooks/useOrganizations';

export const OrganizationContext = createContext<OrganizationsState | null>(null);

export function useOrganizationContext(): OrganizationsState {
  const value = useContext(OrganizationContext);
  if (!value) {
    throw new Error('useOrganizationContext must be used within an OrganizationProvider');
  }
  return value;
}

interface OrganizationProviderProps {
  value: OrganizationsState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function OrganizationProvider({ value, children }: OrganizationProviderProps): JSX.Element {
  return <OrganizationContext.Provider value={value}>{children}</OrganizationContext.Provider>;
}
