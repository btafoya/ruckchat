import { useMemo, type JSX } from 'react';
import { Route, Routes, Navigate, useParams } from 'react-router-dom';
import { createApi } from './api';
import {
  ChannelProvider,
  DirectMessageProvider,
  MessageProvider,
  OrganizationProvider,
  PlatformProvider,
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
  useSettings,
  useTyping,
  useUnread,
  useWebSocket,
} from './hooks';
import { AuthScreen, Settings, Shell } from './components';
import {
  OrgAdminEmoji,
  OrgAdminMembers,
  OrgAdminPermissions,
  OrgAdminRoles,
  OrgAdminSettings,
  OrgAdminShell,
  OrgAdminTeams,
  ServerAdminAdmins,
  ServerAdminAuditLog,
  ServerAdminOrganizations,
  ServerAdminSettings,
  ServerAdminShell,
  ServerAdminUsers,
} from './components/admin';
import type { Platform } from './platform';

interface PlatformShellProps {
  /** Platform-specific integrations for this build. */
  platform: Platform;
}

function AuthenticatedShell({ platform }: { platform: Platform }): JSX.Element {
  const { session } = useSessionContext();
  const settings = useSettings();
  const { apiUrl, notificationsEnabled } = settings;
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
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
    session?.user.display_name,
  );
  const presenceState = usePresence();
  const typingState = useTyping();
  const unreadState = useUnread(conversationId);
  const notificationsState = platform.useNotifications({
    userId: session?.user.id ?? '',
    enabled: session ? !settings.isLoading && notificationsEnabled : false,
    api,
    token: session?.token,
  });
  const realtimeStore = useRealtimeStore(messagesState, presenceState, typingState, unreadState, notificationsState);
  const websocketState = useWebSocket(session?.token, realtimeStore.onEvent, { apiUrl });

  platform.useTray({ unreadCount: unreadState.total, enabled: !!session });
  platform.useDeepLink();

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

function OrgAdminRoute(): JSX.Element {
  const { session } = useSessionContext();
  const { apiUrl } = useSettings();
  const organizationsState = useOrganizations(session?.token, { apiUrl });
  const params = useParams<{ organizationId: string }>();

  return (
    <OrganizationProvider value={organizationsState}>
      <OrgAdminShell key={params.organizationId} />
    </OrganizationProvider>
  );
}

function OrgAdminSettingsRoute(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  if (!organizationId) {
    return <div className="text-gray-400">Organization not selected.</div>;
  }
  return <OrgAdminSettings organizationId={organizationId} />;
}

function OrgAdminRolesRoute(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  if (!organizationId) {
    return <div className="text-gray-400">Organization not selected.</div>;
  }
  return <OrgAdminRoles organizationId={organizationId} />;
}

function OrgAdminPermissionsRoute(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  if (!organizationId) {
    return <div className="text-gray-400">Organization not selected.</div>;
  }
  return <OrgAdminPermissions organizationId={organizationId} />;
}

function OrgAdminEmojiRoute(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  if (!organizationId) {
    return <div className="text-gray-400">Organization not selected.</div>;
  }
  return <OrgAdminEmoji organizationId={organizationId} />;
}

function OrgAdminTeamsRoute(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  if (!organizationId) {
    return <div className="text-gray-400">Organization not selected.</div>;
  }
  return <OrgAdminTeams organizationId={organizationId} />;
}

export default function PlatformShell({ platform }: PlatformShellProps): JSX.Element {
  const sessionState = useSession();

  return (
    <SessionProvider value={sessionState}>
      <PlatformProvider platform={platform}>
        <Routes>
          <Route path="/login" element={<AuthScreen />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/admin/server/*" element={<ServerAdminShell />}>
            <Route index element={<Navigate to="organizations" replace />} />
            <Route path="organizations" element={<ServerAdminOrganizations />} />
            <Route path="users" element={<ServerAdminUsers />} />
            <Route path="admins" element={<ServerAdminAdmins />} />
            <Route path="settings" element={<ServerAdminSettings />} />
            <Route path="audit-log" element={<ServerAdminAuditLog />} />
          </Route>
          <Route path="/org/:organizationId/admin/*" element={<OrgAdminRoute />}>
            <Route index element={<Navigate to="settings" replace />} />
            <Route path="settings" element={<OrgAdminSettingsRoute />} />
            <Route path="members" element={<OrgAdminMembers />} />
            <Route path="roles" element={<OrgAdminRolesRoute />} />
            <Route path="permissions" element={<OrgAdminPermissionsRoute />} />
            <Route path="emoji" element={<OrgAdminEmojiRoute />} />
            <Route path="teams" element={<OrgAdminTeamsRoute />} />
          </Route>
          <Route path="/*" element={<AuthenticatedShell platform={platform} />}>
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
      </PlatformProvider>
    </SessionProvider>
  );
}
