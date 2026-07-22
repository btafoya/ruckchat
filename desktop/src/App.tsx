import type { JSX } from 'react';
import { Route, Routes, Navigate, useParams } from 'react-router-dom';
import {
  ChannelProvider,
  DirectMessageProvider,
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
  useDirectMessages,
  useMessages,
  useOrganizations,
  usePresence,
  useRealtimeStore,
  useSession,
  useTyping,
  useUnread,
  useWebSocket,
} from './hooks';
import { AuthScreen, Shell } from './components';

function AuthenticatedShell(): JSX.Element {
  const { session } = useSessionContext();
  const organizationsState = useOrganizations(session?.token);
  const params = useParams<{
    organizationId?: string;
    channelId?: string;
    dmId?: string;
    messageId?: string;
  }>();
  const organizationId = params.organizationId;
  const channelId = params.channelId;
  const dmId = params.dmId;
  const conversationType = channelId ? 'channel' : dmId ? 'direct_message' : undefined;
  const conversationId = channelId ?? dmId;
  const channelsState = useChannels(session?.token, organizationId);
  const directMessagesState = useDirectMessages(session?.token, organizationId);
  const messagesState = useMessages(
    session?.token,
    conversationType,
    conversationId,
    session?.user.id,
  );
  const presenceState = usePresence();
  const typingState = useTyping();
  const unreadState = useUnread(conversationId);
  const realtimeStore = useRealtimeStore(messagesState, presenceState, typingState, unreadState);
  const websocketState = useWebSocket(session?.token, realtimeStore.onEvent);

  return (
    <OrganizationProvider value={organizationsState}>
      <ChannelProvider value={channelsState}>
        <DirectMessageProvider value={directMessagesState}>
          <MessageProvider value={messagesState}>
            <PresenceProvider value={presenceState}>
              <TypingProvider value={typingState}>
                <RealtimeProvider value={websocketState}>
                  <Shell />
                </RealtimeProvider>
              </TypingProvider>
            </PresenceProvider>
          </MessageProvider>
        </DirectMessageProvider>
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
          <Route
            path="org/:organizationId/channel/:channelId/thread/:messageId"
            element={<div />}
          />
          <Route path="org/:organizationId/dm/:dmId" element={<div />} />
        </Route>
      </Routes>
    </SessionProvider>
  );
}
