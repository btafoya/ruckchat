import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';
import type { OrgMembersState } from '../hooks/useOrgMembers';

export const OrgMemberContext = createContext<OrgMembersState | null>(null);

export function useOrgMemberContext(): OrgMembersState {
  const value = useContext(OrgMemberContext);
  if (!value) {
    throw new Error('useOrgMemberContext must be used within an OrgMemberProvider');
  }
  return value;
}

interface OrgMemberProviderProps {
  value: OrgMembersState;
  children: ReactNode;
}

import type { JSX } from 'react';

export function OrgMemberProvider({ value, children }: OrgMemberProviderProps): JSX.Element {
  return <OrgMemberContext.Provider value={value}>{children}</OrgMemberContext.Provider>;
}
