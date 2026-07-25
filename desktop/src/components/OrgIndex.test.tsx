import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { OrgIndex } from './OrgIndex';
import { OrganizationProvider, ReadStateProvider } from '../context';
import type { Organization } from '../api';

function renderOrgIndex({ organizations, isLoading = false }: { organizations: Organization[]; isLoading?: boolean }) {
  return render(
    <MemoryRouter>
      <OrganizationProvider value={{ organizations, isLoading, error: null, refresh: vi.fn() }}>
        <ReadStateProvider value={{ counts: {}, total: 0, increment: vi.fn(), markRead: vi.fn(), applyRemoteRead: vi.fn(), refresh: vi.fn() }}>
          <OrgIndex />
        </ReadStateProvider>
      </OrganizationProvider>
    </MemoryRouter>,
  );
}

const mockOrg: Organization = {
  id: 'org-1',
  name: 'Acme',
  slug: 'acme',
  owner_id: 'user-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

describe('OrgIndex', () => {
  it('renders a loading state', () => {
    renderOrgIndex({ organizations: [], isLoading: true });
    expect(screen.getByText(/Loading organizations/i)).toBeInTheDocument();
  });

  it('renders an empty state when the user has no organizations', () => {
    renderOrgIndex({ organizations: [] });
    expect(screen.getByText(/Welcome to RuckChat/i)).toBeInTheDocument();
  });

  it('renders a list of organizations', () => {
    renderOrgIndex({
      organizations: [
        mockOrg,
        { ...mockOrg, id: 'org-2', name: 'Globex', slug: 'globex' },
      ],
    });

    expect(screen.getByText('Acme')).toBeInTheDocument();
    expect(screen.getByText('Globex')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Acme' })).toHaveAttribute('href', '/org/org-1');
  });
});
