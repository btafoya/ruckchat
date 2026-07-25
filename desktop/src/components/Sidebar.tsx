import { useCallback, useMemo, useState } from 'react';
import type { JSX } from 'react';
import { NavLink, useNavigate, useParams } from 'react-router-dom';
import { createApi } from '../api';
import type { Channel } from '../api';
import {
  useChannelContext,
  useDirectMessageContext,
  useOrgMemberContext,
  useOrganizationContext,
  useReadStateContext,
  useSessionContext,
  useSettingsContext,
} from '../context';
import { ChannelSettingsModal } from './ChannelSettingsModal';
import { CreateChannelModal } from './CreateChannelModal';
import { StartDmModal } from './StartDmModal';

interface NavBadgeProps {
  count: number;
}

function NavBadge({ count }: NavBadgeProps): JSX.Element | null {
  if (count <= 0) {
    return null;
  }
  return (
    <span className="ml-auto rounded-full bg-accent px-2 py-0.5 text-xs font-semibold text-text-inverse">
      {count > 99 ? '99+' : count}
    </span>
  );
}

interface SidebarProps {
  mobileOpen?: boolean;
  onClose?: () => void;
}

export function Sidebar({ mobileOpen = false, onClose }: SidebarProps): JSX.Element {
  const { session, logout } = useSessionContext();
  const { organizations, isLoading: orgsLoading, error: orgsError } = useOrganizationContext();
  const { channels, isLoading: channelsLoading, error: channelsError } = useChannelContext();
  const {
    conversations,
    isLoading: dmsLoading,
    error: dmsError,
    refresh: refreshDms,
  } = useDirectMessageContext();
  const { members: orgMembers } = useOrgMemberContext();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const navigate = useNavigate();
  const params = useParams();
  const activeOrgId = params.organizationId;
  const activeConversationId = (params.channelId ?? params.dmId) || undefined;
  const { counts } = useReadStateContext();

  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [settingsChannel, setSettingsChannel] = useState<Channel | null>(null);
  const [showStartDm, setShowStartDm] = useState(false);

  const activeOrganization = organizations.find((o) => o.id === activeOrgId);
  const showServerAdmin = session?.user.is_server_admin ?? false;
  const showOrgAdmin =
    activeOrganization !== undefined &&
    session !== null &&
    (session.user.is_server_admin || activeOrganization.owner_id === session.user.id);

  const activeChannels = useMemo(() => channels.filter((c) => !c.archived_at), [channels]);
  const archivedChannels = useMemo(() => channels.filter((c) => c.archived_at), [channels]);

  const memberName = useCallback(
    (userId: string) => orgMembers.find((m) => m.id === userId)?.display_name ?? userId,
    [orgMembers],
  );

  const dmLabels = useMemo(() => {
    return new Map(
      conversations.map((conversation) => {
        const others = conversation.member_ids.filter((id) => id !== session?.user.id);
        const label = others.length > 0 ? others.map(memberName).join(', ') : 'You';
        return [conversation.id, label];
      }),
    );
  }, [conversations, session?.user.id, memberName]);

  const handleNavClick = useCallback(() => {
    if (mobileOpen) {
      onClose?.();
    }
  }, [mobileOpen, onClose]);

  const handleHideDm = useCallback(
    async (conversationId: string) => {
      if (!session) {
        return;
      }
      await api.directMessages.hide(session.token, conversationId);
      await refreshDms();
      if (activeConversationId === conversationId && activeOrganization) {
        navigate(`/org/${activeOrganization.id}/channel`);
      }
    },
    [api, session, refreshDms, activeConversationId, activeOrganization, navigate],
  );

  const asideClass = mobileOpen
    ? 'fixed inset-y-0 left-0 z-20 flex w-64 flex-shrink-0 flex-col border-r border-border bg-surface'
    : 'hidden md:flex w-64 flex-shrink-0 flex-col border-r border-border bg-surface';

  return (
    <aside className={asideClass} aria-label="Navigation">
      <header className="flex items-center justify-between border-b border-border p-4">
        <span className="font-semibold text-text">RuckChat</span>
        <div className="flex items-center gap-2">
          {mobileOpen && (
            <button
              type="button"
              aria-label="Close navigation"
              onClick={onClose}
              className="text-xs text-text-muted hover:text-text md:hidden"
            >
              ✕
            </button>
          )}
          <button
            type="button"
            onClick={() => void logout()}
            className="text-xs text-text-muted hover:text-text"
          >
            Sign out
          </button>
        </div>
      </header>

      <div className="flex flex-col gap-2 p-3">
        <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
          Organizations
        </div>
        {orgsLoading && <div className="text-sm text-text-muted">Loading...</div>}
        {orgsError && <div className="text-sm text-danger">{orgsError}</div>}
        <nav className="flex flex-col gap-1" aria-label="Organizations">
          {organizations.map((org) => (
            <NavLink
              key={org.id}
              to={`/org/${org.id}/channel`}
              onClick={handleNavClick}
              className={({ isActive }) =>
                `rounded-md px-3 py-2 text-sm ${
                  isActive || activeOrgId === org.id
                    ? 'bg-accent text-text-inverse'
                    : 'text-text hover:bg-surface-elevated'
                }`
              }
              end
            >
              {org.name}
            </NavLink>
          ))}
        </nav>
      </div>

      {showServerAdmin && (
        <div className="flex flex-col gap-2 border-t border-border p-3">
          <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
            Server administration
          </div>
          <nav className="flex flex-col gap-1" aria-label="Server administration">
            <NavLink
              to="/admin/server/organizations"
              onClick={handleNavClick}
              className={({ isActive }) =>
                `rounded-md px-3 py-2 text-sm ${
                  isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                }`
              }
            >
              Server Admin
            </NavLink>
          </nav>
        </div>
      )}

      {activeOrganization && (
        <>
          <div className="flex flex-col gap-2 border-t border-border p-3">
            <div className="flex items-center justify-between">
              <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
                {activeOrganization.name} channels
              </div>
              <button
                type="button"
                aria-label="Create channel"
                onClick={() => setShowCreateChannel(true)}
                className="text-sm text-text-muted hover:text-text"
              >
                +
              </button>
            </div>
            {channelsLoading && <div className="text-sm text-text-muted">Loading...</div>}
            {channelsError && <div className="text-sm text-danger">{channelsError}</div>}
            <nav className="flex flex-col gap-1" aria-label="Channels">
              {activeChannels.map((channel) => (
                <div key={channel.id} className="group flex items-center gap-1">
                  <NavLink
                    to={`/org/${activeOrganization.id}/channel/${channel.id}`}
                    onClick={handleNavClick}
                    className={({ isActive }) =>
                      `flex flex-1 items-center rounded-md px-3 py-2 text-sm ${
                        isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                      }`
                    }
                  >
                    <span># {channel.name}</span>
                    <NavBadge count={counts[channel.id] ?? 0} />
                  </NavLink>
                  <button
                    type="button"
                    aria-label={`Channel settings for ${channel.name}`}
                    onClick={() => setSettingsChannel(channel)}
                    className="px-1 text-text-muted opacity-0 hover:text-text group-hover:opacity-100"
                  >
                    ⋯
                  </button>
                </div>
              ))}
            </nav>
            {archivedChannels.length > 0 && (
              <details className="text-sm">
                <summary className="cursor-pointer text-text-muted">Archived</summary>
                <nav className="mt-1 flex flex-col gap-1" aria-label="Archived channels">
                  {archivedChannels.map((channel) => (
                    <button
                      key={channel.id}
                      type="button"
                      onClick={() => setSettingsChannel(channel)}
                      className="flex items-center rounded-md px-3 py-2 text-left text-sm text-text-muted hover:bg-surface-elevated"
                    >
                      # {channel.name}
                    </button>
                  ))}
                </nav>
              </details>
            )}
          </div>

          <div className="flex flex-col gap-2 border-t border-border p-3">
            <div className="flex items-center justify-between">
              <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
                Direct messages
              </div>
              <button
                type="button"
                aria-label="New message"
                onClick={() => setShowStartDm(true)}
                className="text-sm text-text-muted hover:text-text"
              >
                +
              </button>
            </div>
            {dmsLoading && <div className="text-sm text-text-muted">Loading...</div>}
            {dmsError && <div className="text-sm text-danger">{dmsError}</div>}
            <nav className="flex flex-col gap-1" aria-label="Direct messages">
              {conversations.map((conversation) => (
                <div key={conversation.id} className="group flex items-center gap-1">
                  <NavLink
                    to={`/org/${activeOrganization.id}/dm/${conversation.id}`}
                    onClick={handleNavClick}
                    className={({ isActive }) =>
                      `flex flex-1 items-center rounded-md px-3 py-2 text-sm ${
                        isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                      }`
                    }
                  >
                    <span>{dmLabels.get(conversation.id) ?? conversation.id}</span>
                    <NavBadge count={counts[conversation.id] ?? 0} />
                  </NavLink>
                  <button
                    type="button"
                    aria-label="Hide conversation"
                    onClick={() => void handleHideDm(conversation.id)}
                    className="px-1 text-text-muted opacity-0 hover:text-text group-hover:opacity-100"
                  >
                    ✕
                  </button>
                </div>
              ))}
            </nav>
          </div>

          {showOrgAdmin && (
            <div className="flex flex-col gap-2 border-t border-border p-3">
              <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
                {activeOrganization.name} administration
              </div>
              <nav className="flex flex-col gap-1" aria-label="Organization administration">
                <NavLink
                  to={`/org/${activeOrganization.id}/admin/settings`}
                  onClick={handleNavClick}
                  className={({ isActive }) =>
                    `rounded-md px-3 py-2 text-sm ${
                      isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                    }`
                  }
                >
                  Admin
                </NavLink>
              </nav>
            </div>
          )}
        </>
      )}

      <div className="mt-auto border-t border-border p-3">
        <div className="flex items-center justify-between">
          <div className="text-sm text-text">{session?.user.display_name ?? session?.user.email}</div>
          <NavLink
            to="/settings"
            className="text-xs text-text-muted hover:text-text"
            aria-label="Settings"
          >
            Settings
          </NavLink>
        </div>
      </div>

      {showCreateChannel && activeOrganization && (
        <CreateChannelModal
          organizationId={activeOrganization.id}
          onClose={() => setShowCreateChannel(false)}
        />
      )}
      {settingsChannel && (
        <ChannelSettingsModal channel={settingsChannel} onClose={() => setSettingsChannel(null)} />
      )}
      {showStartDm && activeOrganization && (
        <StartDmModal organizationId={activeOrganization.id} onClose={() => setShowStartDm(false)} />
      )}
    </aside>
  );
}
