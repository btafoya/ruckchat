import { useCallback, useMemo } from 'react';
import type { JSX } from 'react';
import { NavLink, useParams } from 'react-router-dom';
import {
  useChannelContext,
  useDirectMessageContext,
  useOrganizationContext,
  useSessionContext,
} from '../context';
import { useUnread } from '../hooks';

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
  const { conversations, isLoading: dmsLoading, error: dmsError } = useDirectMessageContext();
  const params = useParams();
  const activeOrgId = params.organizationId;
  const activeConversationId = (params.channelId ?? params.dmId) || undefined;
  const { counts } = useUnread(activeConversationId);

  const activeOrganization = organizations.find((o) => o.id === activeOrgId);
  const showServerAdmin = session?.user.is_server_admin ?? false;
  const showOrgAdmin =
    activeOrganization !== undefined &&
    session !== null &&
    (session.user.is_server_admin || activeOrganization.owner_id === session.user.id);

  const dmLabels = useMemo(() => {
    return new Map(
      conversations.map((conversation) => {
        const others = conversation.member_ids.filter((id) => id !== session?.user.id);
        const label = others.length > 0 ? others.join(', ') : 'You';
        return [conversation.id, label];
      }),
    );
  }, [conversations, session?.user.id]);

  const handleNavClick = useCallback(() => {
    if (mobileOpen) {
      onClose?.();
    }
  }, [mobileOpen, onClose]);

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
            <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
              {activeOrganization.name} channels
            </div>
            {channelsLoading && <div className="text-sm text-text-muted">Loading...</div>}
            {channelsError && <div className="text-sm text-danger">{channelsError}</div>}
            <nav className="flex flex-col gap-1" aria-label="Channels">
              {channels.map((channel) => (
                <NavLink
                  key={channel.id}
                  to={`/org/${activeOrganization.id}/channel/${channel.id}`}
                  onClick={handleNavClick}
                  className={({ isActive }) =>
                    `flex items-center rounded-md px-3 py-2 text-sm ${
                      isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                    }`
                  }
                >
                  <span># {channel.name}</span>
                  <NavBadge count={counts[channel.id] ?? 0} />
                </NavLink>
              ))}
            </nav>
          </div>

          <div className="flex flex-col gap-2 border-t border-border p-3">
            <div className="text-xs font-semibold uppercase tracking-wider text-text-muted">
              Direct messages
            </div>
            {dmsLoading && <div className="text-sm text-text-muted">Loading...</div>}
            {dmsError && <div className="text-sm text-danger">{dmsError}</div>}
            <nav className="flex flex-col gap-1" aria-label="Direct messages">
              {conversations.map((conversation) => (
                <NavLink
                  key={conversation.id}
                  to={`/org/${activeOrganization.id}/dm/${conversation.id}`}
                  onClick={handleNavClick}
                  className={({ isActive }) =>
                    `flex items-center rounded-md px-3 py-2 text-sm ${
                      isActive ? 'bg-accent text-text-inverse' : 'text-text hover:bg-surface-elevated'
                    }`
                  }
                >
                  <span>{dmLabels.get(conversation.id) ?? conversation.id}</span>
                  <NavBadge count={counts[conversation.id] ?? 0} />
                </NavLink>
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
    </aside>
  );
}
