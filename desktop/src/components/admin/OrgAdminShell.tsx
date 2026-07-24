import type { JSX } from 'react';
import { NavLink, Navigate, Outlet, useLocation, useParams } from 'react-router-dom';
import { useOrganizationContext, useSessionContext } from '../../context';

const tabs = [
  { path: 'settings', label: 'Settings' },
  { path: 'members', label: 'Members' },
  { path: 'roles', label: 'Roles' },
  { path: 'permissions', label: 'Permissions' },
  { path: 'emoji', label: 'Emoji' },
  { path: 'teams', label: 'Teams' },
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
      <header className="border-b border-border bg-surface px-6 py-4">
        <h1 className="text-lg font-semibold">
          {organization ? `${organization.name} Administration` : 'Organization Administration'}
        </h1>
      </header>
      <div className="flex flex-1 overflow-hidden">
        <nav
          className="flex w-48 flex-shrink-0 flex-col gap-1 border-r border-border bg-surface p-3"
          aria-label="Org admin"
        >
          {tabs.map((tab) => (
            <NavLink
              key={tab.path}
              to={tab.path}
              className={({ isActive }) =>
                `rounded-md px-3 py-2 text-sm ${
                  isActive
                    ? 'bg-accent text-text-inverse'
                    : 'text-text hover:bg-surface-elevated'
                }`
              }
            >
              {tab.label}
            </NavLink>
          ))}
        </nav>
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
