import type { JSX } from 'react';
import { Navigate, Outlet, useParams } from 'react-router-dom';
import { useSessionContext } from '../context';
import { useChannels, useOrganizations } from '../hooks';
import { ChannelProvider, OrganizationProvider } from '../context';
import { Sidebar } from './Sidebar';
import { MessagePane } from './MessagePane';

export function Shell(): JSX.Element {
  const { session, isLoading } = useSessionContext();
  const organizationsState = useOrganizations(session?.token);
  const params = useParams();
  const activeOrganizationId = params.organizationId;
  const channelsState = useChannels(session?.token, activeOrganizationId);

  if (isLoading) {
    return <div className="flex h-screen items-center justify-center bg-gray-900 text-white">Loading...</div>;
  }

  if (!session) {
    return <Navigate to="/login" replace />;
  }

  return (
    <OrganizationProvider value={organizationsState}>
      <ChannelProvider value={channelsState}>
        <div className="flex h-screen w-screen overflow-hidden bg-gray-900 text-white">
          <Sidebar />
          <div className="flex flex-1 flex-col overflow-hidden">
            <MessagePane />
            <Outlet />
          </div>
        </div>
      </ChannelProvider>
    </OrganizationProvider>
  );
}
