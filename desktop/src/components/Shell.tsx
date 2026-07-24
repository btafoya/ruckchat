import { useCallback, useState, type JSX } from 'react';
import { Navigate, Outlet } from 'react-router-dom';
import { useSessionContext } from '../context';
import { Sidebar } from './Sidebar';
import { MessagePane } from './MessagePane';

export function Shell(): JSX.Element {
  const { session, isLoading } = useSessionContext();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const openSidebar = useCallback(() => setSidebarOpen(true), []);
  const closeSidebar = useCallback(() => setSidebarOpen(false), []);

  if (isLoading) {
    return <div className="flex h-screen items-center justify-center bg-bg text-text">Loading...</div>;
  }

  if (!session) {
    return <Navigate to="/login" replace />;
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg text-text">
      <Sidebar mobileOpen={sidebarOpen} onClose={closeSidebar} />

      {sidebarOpen && (
        <button
          type="button"
          aria-label="Close navigation"
          className="fixed inset-0 z-10 bg-overlay md:hidden"
          onClick={closeSidebar}
        />
      )}

      <div className="relative flex flex-1 flex-col overflow-hidden">
        <button
          type="button"
          aria-label="Open navigation"
          onClick={openSidebar}
          className="absolute left-2 top-2 z-20 rounded-md bg-surface px-2 py-1 text-sm md:hidden"
        >
          ☰
        </button>
        <MessagePane />
        <Outlet />
      </div>
    </div>
  );
}
