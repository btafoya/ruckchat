import { type JSX } from 'react';
import { Link } from 'react-router-dom';
import { useOrganizationContext } from '../context';

export function OrgIndex(): JSX.Element {
  const { organizations, isLoading } = useOrganizationContext();

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center bg-bg p-6 text-text">
        <span className="text-sm text-text-muted">Loading organizations...</span>
      </div>
    );
  }

  if (organizations.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center bg-bg p-6 text-center text-text">
        <h1 className="mb-2 text-2xl font-semibold">Welcome to RuckChat</h1>
        <p className="max-w-md text-sm text-text-muted">
          You are not a member of any organization yet. Ask an administrator for an invitation.
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto bg-bg p-6 text-text">
      <header className="mb-6">
        <h1 className="text-2xl font-semibold">Organizations</h1>
        <p className="text-sm text-text-muted">Choose an organization to view its conversations.</p>
      </header>

      <nav aria-label="Organizations" className="space-y-1">
        {organizations.map((org) => (
          <Link
            key={org.id}
            to={`/org/${org.id}`}
            className="flex items-center justify-between rounded-md border border-border bg-surface px-4 py-3 hover:bg-surface-elevated"
          >
            <span className="text-sm font-medium">{org.name}</span>
          </Link>
        ))}
      </nav>
    </div>
  );
}
