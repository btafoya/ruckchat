import type { JSX } from 'react';
import { NavLink, Navigate, Outlet, useLocation } from 'react-router-dom';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faBuilding,
  faClipboardList,
  faGear,
  faUserShield,
  faUsers,
} from '@fortawesome/free-solid-svg-icons';
import { useSessionContext } from '../../context';

const tabs = [
  { path: '/admin/server/organizations', label: 'Organizations', icon: faBuilding },
  { path: '/admin/server/users', label: 'Users', icon: faUsers },
  { path: '/admin/server/admins', label: 'Admins', icon: faUserShield },
  { path: '/admin/server/settings', label: 'Settings', icon: faGear },
  { path: '/admin/server/audit-log', label: 'Audit Log', icon: faClipboardList },
];

export function ServerAdminShell(): JSX.Element {
  const { session, isLoading } = useSessionContext();
  const location = useLocation();

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

  if (!session.user.is_server_admin) {
    return (
      <div className="flex h-screen items-center justify-center bg-bg text-text">
        <div className="text-center">
          <h1 className="text-2xl font-bold">Forbidden</h1>
          <p className="mt-2 text-text-muted">
            Server administrator access is required.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-bg text-text">
      <header className="flex items-center justify-between border-b border-border bg-surface px-4 py-3">
        <nav className="flex items-center gap-1 overflow-x-auto" aria-label="Server admin">
          {tabs.map((tab) => (
            <NavLink
              key={tab.path}
              to={tab.path}
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
          <h1 className="text-lg font-semibold">Server Administration</h1>
          <NavLink to="/" className="text-sm text-text-muted hover:text-text">
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
