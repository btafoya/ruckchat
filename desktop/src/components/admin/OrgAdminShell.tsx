import type { JSX } from 'react';
import { NavLink, Navigate, Outlet, useLocation, useParams } from 'react-router-dom';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faFaceSmile,
  faGear,
  faKey,
  faPeopleGroup,
  faUserLock,
  faUsers,
} from '@fortawesome/free-solid-svg-icons';
import { useOrganizationContext, useSessionContext } from '../../context';
import { getLastConversation } from '../../lastConversation';

const tabs = [
  { path: 'settings', label: 'Settings', icon: faGear },
  { path: 'members', label: 'Members', icon: faUsers },
  { path: 'roles', label: 'Roles', icon: faUserLock },
  { path: 'permissions', label: 'Permissions', icon: faKey },
  { path: 'emoji', label: 'Emoji', icon: faFaceSmile },
  { path: 'teams', label: 'Teams', icon: faPeopleGroup },
];

export function OrgAdminShell(): JSX.Element {
  const { session, isLoading } = useSessionContext();
  const { organizations } = useOrganizationContext();
  const params = useParams();
  const location = useLocation();
  const organizationId = params.organizationId;

  const organization = organizations.find((o) => o.id === organizationId);
  const canAdmin =
    !!session &&
    (session.user.is_server_admin || organization?.owner_id === session.user.id);

  const backTo = (() => {
    if (!organizationId) return '/';
    const last = getLastConversation(organizationId);
    if (last?.type === 'channel') return `/org/${organizationId}/channel/${last.id}`;
    if (last?.type === 'dm') return `/org/${organizationId}/dm/${last.id}`;
    return `/org/${organizationId}/channel`;
  })();

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center bg-bg text-text">
        Loading...
      </div>
    );
  }

  if (!session) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  if (!canAdmin) {
    return (
      <div className="flex h-screen items-center justify-center bg-bg text-text">
        <div className="text-center">
          <h1 className="text-2xl font-bold">Forbidden</h1>
          <p className="mt-2 text-text-muted">
            Organization administrator access is required.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-bg text-text">
      <header className="flex items-center justify-between border-b border-border bg-surface px-4 py-3">
        <nav className="flex items-center gap-1 overflow-x-auto" aria-label="Org admin">
          {tabs.map((tab) => (
            <NavLink
              key={tab.path}
              to={`/org/${organizationId}/admin/${tab.path}`}
              title={tab.label}
              aria-label={tab.label}
              className={({ isActive }) =>
                `flex flex-shrink-0 items-center justify-center rounded-md p-2 text-lg ${
                  isActive
                    ? 'bg-accent text-text-inverse'
                    : 'text-text hover:bg-surface-elevated'
                }`
              }
            >
              <FontAwesomeIcon icon={tab.icon} />
            </NavLink>
          ))}
        </nav>
        <div className="flex flex-shrink-0 items-center gap-3 pl-4">
          <h1 className="text-lg font-semibold">
            {organization ? `${organization.name} Administration` : 'Organization Administration'}
          </h1>
          <NavLink to={backTo} className="text-sm text-text-muted hover:text-text">
            Back
          </NavLink>
        </div>
      </header>
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
