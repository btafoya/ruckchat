import type { JSX } from 'react';
import { NavLink, useParams } from 'react-router-dom';
import { useChannelContext, useOrganizationContext, useSessionContext } from '../context';

export function Sidebar(): JSX.Element {
  const { session, logout } = useSessionContext();
  const { organizations, isLoading: orgsLoading, error: orgsError } = useOrganizationContext();
  const { channels, isLoading: channelsLoading, error: channelsError } = useChannelContext();
  const params = useParams();
  const activeOrgId = params.organizationId;

  const activeOrganization = organizations.find((o) => o.id === activeOrgId);

  return (
    <aside className="flex w-64 flex-shrink-0 flex-col border-r border-gray-700 bg-gray-800" aria-label="Navigation">
      <header className="flex items-center justify-between border-b border-gray-700 p-4">
        <span className="font-semibold text-white">RuckChat</span>
        <button
          type="button"
          onClick={() => void logout()}
          className="text-xs text-gray-400 hover:text-white"
        >
          Sign out
        </button>
      </header>

      <div className="flex flex-col gap-2 p-3">
        <div className="text-xs font-semibold uppercase tracking-wider text-gray-400">
          Organizations
        </div>
        {orgsLoading && <div className="text-sm text-gray-400">Loading...</div>}
        {orgsError && <div className="text-sm text-red-400">{orgsError}</div>}
        <nav className="flex flex-col gap-1" aria-label="Organizations">
          {organizations.map((org) => (
            <NavLink
              key={org.id}
              to={`/org/${org.id}/channel`}
              className={({ isActive }) =>
                `rounded-md px-3 py-2 text-sm ${
                  isActive || activeOrgId === org.id
                    ? 'bg-green-700 text-white'
                    : 'text-gray-300 hover:bg-gray-700'
                }`
              }
              end
            >
              {org.name}
            </NavLink>
          ))}
        </nav>
      </div>

      {activeOrganization && (
        <div className="flex flex-col gap-2 border-t border-gray-700 p-3">
          <div className="text-xs font-semibold uppercase tracking-wider text-gray-400">
            {activeOrganization.name} channels
          </div>
          {channelsLoading && <div className="text-sm text-gray-400">Loading...</div>}
          {channelsError && <div className="text-sm text-red-400">{channelsError}</div>}
          <nav className="flex flex-col gap-1" aria-label="Channels">
            {channels.map((channel) => (
              <NavLink
                key={channel.id}
                to={`/org/${activeOrganization.id}/channel/${channel.id}`}
                className={({ isActive }) =>
                  `rounded-md px-3 py-2 text-sm ${
                    isActive ? 'bg-green-700 text-white' : 'text-gray-300 hover:bg-gray-700'
                  }`
                }
              >
                # {channel.name}
              </NavLink>
            ))}
          </nav>
        </div>
      )}

      <div className="mt-auto border-t border-gray-700 p-3">
        <div className="text-sm text-gray-300">{session?.user.display_name ?? session?.user.email}</div>
      </div>
    </aside>
  );
}
