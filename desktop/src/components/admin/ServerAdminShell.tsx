import type { JSX } from 'react';
import { NavLink, Navigate, Outlet, useLocation } from 'react-router-dom';
import { useSessionContext } from '../../context';

const tabs = [
  { path: '/admin/server/organizations', label: 'Organizations' },
  { path: '/admin/server/users', label: 'Users' },
  { path: '/admin/server/admins', label: 'Admins' },
  { path: '/admin/server/settings', label: 'Settings' },
  { path: '/admin/server/audit-log', label: 'Audit Log' },
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
      <header className="border-b border-border bg-surface px-6 py-4">
        <h1 className="text-lg font-semibold">Server Administration</h1>
      </header>
      <div className="flex flex-1 overflow-hidden">
        <nav
          className="flex w-48 flex-shrink-0 flex-col gap-1 border-r border-border bg-surface p-3"
          aria-label="Server admin"
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
