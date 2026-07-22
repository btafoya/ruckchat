import type { JSX } from 'react';
import { Route, Routes, Navigate, useParams } from 'react-router-dom';
import {
  ChannelProvider,
  MessageProvider,
  OrganizationProvider,
  PresenceProvider,
  RealtimeProvider,
  SessionProvider,
  TypingProvider,
  useSessionContext,
} from './context';
import {
  useChannels,
  useMessages,
  useOrganizations,
  usePresence,
  useRealtimeStore,
  useSession,
  useTyping,
  useWebSocket,
} from './hooks';
import { AuthScreen, Shell } from './components';

function AuthenticatedShell(): JSX.Element {
  const { session } = useSessionContext();
  const organizationsState = useOrganizations(session?.token);
  const params = useParams();
  const organizationId = params.organizationId;
  const channelId = params.channelId;
  const channelsState = useChannels(session?.token, organizationId);
  const messagesState = useMessages(
    session?.token,
    channelId ? 'channel' : undefined,
    channelId,
  );
  const presenceState = usePresence();
  const typingState = useTyping();
  const realtimeStore = useRealtimeStore(messagesState, presenceState, typingState);
  const websocketState = useWebSocket(session?.token, realtimeStore.onEvent);

  return (
    <OrganizationProvider value={organizationsState}>
      <ChannelProvider value={channelsState}>
        <MessageProvider value={messagesState}>
          <PresenceProvider value={presenceState}>
            <TypingProvider value={typingState}>
              <RealtimeProvider value={websocketState}>
                <Shell />
              </RealtimeProvider>
            </TypingProvider>
          </PresenceProvider>
        </MessageProvider>
      </ChannelProvider>
    </OrganizationProvider>
  );
}

export default function App(): JSX.Element {
  const sessionState = useSession();

  return (
    <SessionProvider value={sessionState}>
      <Routes>
        <Route path="/login" element={<AuthScreen />} />
        <Route path="/*" element={<AuthenticatedShell />}>
          <Route index element={<Navigate to="/org" replace />} />
          <Route path="org" element={<div />} />
          <Route path="org/:organizationId/channel" element={<div />} />
          <Route path="org/:organizationId/channel/:channelId" element={<div />} />
        </Route>
      </Routes>
    </SessionProvider>
  );
}
