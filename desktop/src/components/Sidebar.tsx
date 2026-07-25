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

interface NarrowBadgeProps {
  count: number;
}

function NarrowBadge({ count }: NarrowBadgeProps): JSX.Element | null {
  if (count <= 0) {
    return null;
  }
  return (
    <span className="absolute -right-0.5 -top-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-accent px-1 text-[10px] font-semibold text-text-inverse">
      {count > 99 ? '99+' : count}
    </span>
  );
}

function initials(label: string): string {
  const trimmed = label.trim();
  const parts = trimmed.split(/[\s,]+/).filter(Boolean);
  if (parts.length === 0) {
    return '?';
  }
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

interface SidebarProps {
  mobileOpen?: boolean;
  onClose?: () => void;
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

export function Sidebar({
  mobileOpen = false,
  onClose,
  collapsed = false,
  onToggleCollapse,
}: SidebarProps): JSX.Element {
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

  const isNarrow = collapsed && !mobileOpen;

  const asideClass = mobileOpen
    ? 'fixed inset-y-0 left-0 z-20 flex w-64 flex-shrink-0 flex-col border-r border-border bg-surface'
    : collapsed
      ? 'hidden md:flex w-16 flex-shrink-0 flex-col border-r border-border bg-surface'
      : 'hidden md:flex w-64 flex-shrink-0 flex-col border-r border-border bg-surface';

  const userInitial = initials(session?.user.display_name ?? session?.user.email ?? 'User');

  return (
    <aside className={asideClass} aria-label="Navigation">
      <header
        className={`flex items-center border-b border-border ${isNarrow ? 'justify-center p-2' : 'justify-between p-4'}`}
      >
        {!isNarrow && <span className="font-semibold text-text">RuckChat</span>}
        <div className="flex items-center gap-2">
          {!mobileOpen && (
            <button
              type="button"
              aria-label={isNarrow ? 'Expand sidebar' : 'Collapse sidebar'}
              onClick={onToggleCollapse}
              className="text-xs text-text-muted hover:text-text"
            >
              {isNarrow ? '»' : '«'}
            </button>
          )}
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
          {!isNarrow && (
            <button
              type="button"
              onClick={() => void logout()}
              className="text-xs text-text-muted hover:text-text"
            >
              Sign out
            </button>
          )}
        </div>
      </header>

      <div className={`flex flex-col gap-2 ${isNarrow ? 'p-2' : 'p-3'}`}>
        {!isNarrow && (
          <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
            Organizations
          </div>
        )}
        {orgsLoading && <div className="text-sm text-text-muted">Loading...</div>}
        {orgsError && <div className="text-sm text-danger">{orgsError}</div>}
        <nav
          className={`flex ${isNarrow ? 'flex-col items-center gap-2' : 'flex-col gap-1'}`}
          aria-label="Organizations"
        >
          {organizations.map((org) => (
            <NavLink
              key={org.id}
              to={`/org/${org.id}/channel`}
              title={org.name}
              onClick={handleNavClick}
              className={({ isActive }) =>
                `relative flex items-center justify-center rounded-md ${
                  isNarrow ? 'h-9 w-9 text-sm' : 'px-3 py-2 text-sm'
                } ${
                  isActive || activeOrgId === org.id
                    ? 'bg-accent text-text-inverse'
                    : 'text-text hover:bg-surface-elevated'
                }`
              }
              end
            >
              {isNarrow ? org.name.slice(0, 2).toUpperCase() : org.name}
            </NavLink>
          ))}
        </nav>
      </div>

      {showServerAdmin && (
        <div className={`flex flex-col gap-2 border-t border-border ${isNarrow ? 'p-2' : 'p-3'}`}>
          {!isNarrow && (
            <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
              Server administration
            </div>
          )}
          <nav
            className={`flex ${isNarrow ? 'flex-col items-center gap-2' : 'flex-col gap-1'}`}
            aria-label="Server administration"
          >
            <NavLink
              to="/admin/server/organizations"
              title="Server Admin"
              onClick={handleNavClick}
              className={({ isActive }) =>
                `relative flex items-center justify-center rounded-md ${
                  isNarrow ? 'h-9 w-9 text-sm' : 'px-3 py-2 text-sm'
                } ${
                  isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                }`
              }
            >
              {isNarrow ? 'SA' : 'Server Admin'}
            </NavLink>
          </nav>
        </div>
      )}

      {activeOrganization && (
        <>
          <div className={`flex flex-col gap-2 border-t border-border ${isNarrow ? 'p-2' : 'p-3'}`}>
            {!isNarrow && (
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
            )}
            {isNarrow && (
              <button
                type="button"
                aria-label="Create channel"
                onClick={() => setShowCreateChannel(true)}
                className="flex h-9 w-9 items-center justify-center rounded-md text-sm text-text-muted hover:bg-surface-elevated hover:text-text"
              >
                +
              </button>
            )}
            {channelsLoading && <div className="text-sm text-text-muted">Loading...</div>}
            {channelsError && <div className="text-sm text-danger">{channelsError}</div>}
            <nav
              className={`flex ${isNarrow ? 'flex-col items-center gap-2' : 'flex-col gap-1'}`}
              aria-label="Channels"
            >
              {activeChannels.map((channel) => (
                <div key={channel.id} className={`group flex items-center ${isNarrow ? 'relative' : 'gap-1'}`}>
                  <NavLink
                    to={`/org/${activeOrganization.id}/channel/${channel.id}`}
                    title={channel.name}
                    onClick={handleNavClick}
                    className={({ isActive }) =>
                      `relative flex items-center justify-center rounded-md ${
                        isNarrow ? 'h-9 w-9 text-sm' : 'flex-1 px-3 py-2 text-sm'
                      } ${
                        isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                      }`
                    }
                  >
                    {isNarrow ? channel.name.slice(0, 2).toUpperCase() : <span># {channel.name}</span>}
                    <NarrowBadge count={counts[channel.id] ?? 0} />
                    {!isNarrow && <NavBadge count={counts[channel.id] ?? 0} />}
                  </NavLink>
                  {!isNarrow && (
                    <button
                      type="button"
                      aria-label={`Channel settings for ${channel.name}`}
                      onClick={() => setSettingsChannel(channel)}
                      className="px-1 text-text-muted opacity-0 hover:text-text group-hover:opacity-100"
                    >
                      ⋯
                    </button>
                  )}
                </div>
              ))}
            </nav>
          </div>

          <div className={`flex flex-col gap-2 border-t border-border ${isNarrow ? 'p-2' : 'p-3'}`}>
            {!isNarrow && (
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
            )}
            {isNarrow && (
              <button
                type="button"
                aria-label="New message"
                onClick={() => setShowStartDm(true)}
                className="flex h-9 w-9 items-center justify-center rounded-md text-sm text-text-muted hover:bg-surface-elevated hover:text-text"
              >
                +
              </button>
            )}
            {dmsLoading && <div className="text-sm text-text-muted">Loading...</div>}
            {dmsError && <div className="text-sm text-danger">{dmsError}</div>}
            <nav
              className={`flex ${isNarrow ? 'flex-col items-center gap-2' : 'flex-col gap-1'}`}
              aria-label="Direct messages"
            >
              {conversations.map((conversation) => (
                <div key={conversation.id} className={`group flex items-center ${isNarrow ? 'relative' : 'gap-1'}`}>
                  <NavLink
                    to={`/org/${activeOrganization.id}/dm/${conversation.id}`}
                    title={dmLabels.get(conversation.id) ?? conversation.id}
                    onClick={handleNavClick}
                    className={({ isActive }) =>
                      `relative flex items-center justify-center rounded-md ${
                        isNarrow ? 'h-9 w-9 text-sm' : 'flex-1 px-3 py-2 text-sm'
                      } ${
                        isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                      }`
                    }
                  >
                    {isNarrow
                      ? initials(dmLabels.get(conversation.id) ?? conversation.id)
                      : dmLabels.get(conversation.id) ?? conversation.id}
                    <NarrowBadge count={counts[conversation.id] ?? 0} />
                    {!isNarrow && <NavBadge count={counts[conversation.id] ?? 0} />}
                  </NavLink>
                  {!isNarrow && (
                    <button
                      type="button"
                      aria-label="Hide conversation"
                      onClick={() => void handleHideDm(conversation.id)}
                      className="px-1 text-text-muted opacity-0 hover:text-text group-hover:opacity-100"
                    >
                      ✕
                    </button>
                  )}
                </div>
              ))}
            </nav>
          </div>

          {showOrgAdmin && (
            <div className={`flex flex-col gap-2 border-t border-border ${isNarrow ? 'p-2' : 'p-3'}`}>
              {!isNarrow && (
                <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
                  {activeOrganization.name} administration
                </div>
              )}
              <nav
                className={`flex ${isNarrow ? 'flex-col items-center gap-2' : 'flex-col gap-1'}`}
                aria-label="Organization administration"
              >
                <NavLink
                  to={`/org/${activeOrganization.id}/admin/settings`}
                  title="Admin"
                  onClick={handleNavClick}
                  className={({ isActive }) =>
                    `relative flex items-center justify-center rounded-md ${
                      isNarrow ? 'h-9 w-9 text-sm' : 'px-3 py-2 text-sm'
                    } ${
                      isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                    }`
                  }
                >
                  {isNarrow ? 'OA' : 'Admin'}
                </NavLink>
              </nav>
            </div>
          )}
        </>
      )}

      <div className={`mt-auto border-t border-border ${isNarrow ? 'p-2' : 'p-3'}`}>
        {isNarrow ? (
          <div className="flex flex-col items-center gap-2">
            <div
              className="flex h-9 w-9 items-center justify-center rounded-md bg-surface-elevated text-sm font-semibold text-text"
              title={session?.user.display_name ?? session?.user.email}
            >
              {userInitial}
            </div>
            <NavLink
              to="/settings"
              className="flex h-9 w-9 items-center justify-center rounded-md text-sm text-text-muted hover:bg-surface-elevated hover:text-text"
              aria-label="Settings"
              title="Settings"
            >
              ⚙
            </NavLink>
          </div>
        ) : (
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
        )}
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
