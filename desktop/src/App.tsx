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
  useDeepLink,
  useDirectMessages,
  useMessages,
  useNotifications,
  useOrganizations,
  usePresence,
  useRealtimeStore,
  useSession,
  useSettings,
  useTray,
  useTyping,
  useUnread,
  useWebSocket,
} from './hooks';
import { AuthScreen, Settings, Shell } from './components';

function AuthenticatedShell(): JSX.Element {
  const { session } = useSessionContext();
  const settings = useSettings();
  const { apiUrl, notificationsEnabled } = settings;
  const organizationsState = useOrganizations(session?.token, { apiUrl });
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
  const channelsState = useChannels(session?.token, organizationId, { apiUrl });
  const directMessagesState = useDirectMessages(session?.token, organizationId, { apiUrl });
  const messagesState = useMessages(
    session?.token,
    conversationType,
    conversationId,
    session?.user.id,
    { apiUrl },
  );
  const presenceState = usePresence();
  const typingState = useTyping();
  const unreadState = useUnread(conversationId);
  const notificationsState = useNotifications({
    userId: session?.user.id ?? '',
    enabled: session ? !settings.isLoading && notificationsEnabled : false,
  });
  const realtimeStore = useRealtimeStore(messagesState, presenceState, typingState, unreadState, notificationsState);
  const websocketState = useWebSocket(session?.token, realtimeStore.onEvent, { apiUrl });

  useTray({ unreadCount: unreadState.total, enabled: !!session });
  useDeepLink();

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
        <Route path="/settings" element={<Settings />} />
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
