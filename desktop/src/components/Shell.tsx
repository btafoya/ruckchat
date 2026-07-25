import { useCallback, useState, type FormEvent, type JSX } from 'react';
import { Navigate, Outlet, useLocation, useNavigate, useParams } from 'react-router-dom';
import { useSessionContext, useSettingsContext } from '../context';
import { Sidebar } from './Sidebar';
import { MessagePane } from './MessagePane';

export function Shell(): JSX.Element {
  const { session, isLoading } = useSessionContext();
  const { sidebarCollapsed, setSidebarCollapsed } = useSettingsContext();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [searchInput, setSearchInput] = useState('');
  const location = useLocation();
  const navigate = useNavigate();
  const { organizationId } = useParams<{ organizationId?: string }>();
  const isSearchRoute = location.pathname.endsWith('/search');

  const openSidebar = useCallback(() => setSidebarOpen(true), []);
  const closeSidebar = useCallback(() => setSidebarOpen(false), []);
  const toggleCollapsed = useCallback(
    () => setSidebarCollapsed(!sidebarCollapsed),
    [sidebarCollapsed, setSidebarCollapsed],
  );

  const handleSearchSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (!organizationId || !searchInput.trim()) {
        return;
      }
      navigate(`/org/${organizationId}/search?q=${encodeURIComponent(searchInput.trim())}`);
    },
    [navigate, organizationId, searchInput],
  );

  if (isLoading) {
    return <div className="flex h-screen items-center justify-center bg-bg text-text">Loading...</div>;
  }

  if (!session) {
    return <Navigate to="/login" replace />;
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg text-text">
      <Sidebar
        mobileOpen={sidebarOpen}
        onClose={closeSidebar}
        collapsed={sidebarCollapsed}
        onToggleCollapse={toggleCollapsed}
      />

      {sidebarOpen && (
        <button
          type="button"
          aria-label="Close navigation"
          className="fixed inset-0 z-10 bg-overlay md:hidden"
          onClick={closeSidebar}
        />
      )}

      <div className="relative flex flex-1 flex-col overflow-hidden">
        <div className="flex items-center gap-2 border-b border-border bg-surface px-3 py-2 pl-12 md:pl-3">
          <button
            type="button"
            aria-label="Open navigation"
            onClick={openSidebar}
            className="absolute left-2 top-2 z-20 rounded-md bg-surface px-2 py-1 text-sm md:hidden"
          >
            ☰
          </button>
          {organizationId && (
            <form onSubmit={handleSearchSubmit} className="flex-1">
              <input
                type="search"
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                placeholder="Search messages, channels, people, files..."
                aria-label="Search"
                className="w-full max-w-md rounded-md border border-border bg-bg px-3 py-1.5 text-sm text-text placeholder:text-text-muted focus:border-accent focus:outline-none"
              />
            </form>
          )}
        </div>
        {isSearchRoute ? <Outlet /> : <MessagePane />}
      </div>
    </div>
  );
}
