import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { OrgHome } from './OrgHome';
import {
  ChannelProvider,
  DirectMessageProvider,
  OrgMemberProvider,
  OrganizationProvider,
  ReadStateProvider,
  SessionProvider,
} from '../context';
import type { Channel, DirectMessageConversation, Organization, User } from '../api';
import type { Session } from '../hooks/useSession';

const mockUser: User = {
  id: 'user-1',
  email: 'alice@example.com',
  display_name: 'Alice',
  avatar_url: null,
  is_server_admin: false,
  theme: 'system',
};

const mockUser2: User = {
  id: 'user-2',
  email: 'bob@example.com',
  display_name: 'Bob',
  avatar_url: null,
  is_server_admin: false,
  theme: 'system',
};

const mockSession: Session = {
  token: 'test-token',
  user: mockUser,
};

const mockOrganization: Organization = {
  id: 'org-1',
  name: 'Acme',
  slug: 'acme',
  owner_id: mockUser.id,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

const mockChannel: Channel = {
  id: 'chan-1',
  organization_id: 'org-1',
  name: 'general',
  topic: null,
  purpose: null,
  is_private: false,
  created_by: mockUser.id,
  created_at: '2026-01-01T00:00:00Z',
  archived_at: null,
};

function renderOrgHome({
  channels = [],
  conversations = [],
  counts = {},
}: {
  channels?: Channel[];
  conversations?: DirectMessageConversation[];
  counts?: Record<string, number>;
} = {}) {
  return render(
    <MemoryRouter initialEntries={['/org/org-1']}>
      <SessionProvider value={{ session: mockSession, isLoading: false, error: null, login: vi.fn(), register: vi.fn(), logout: vi.fn() }}>
        <OrganizationProvider value={{ organizations: [mockOrganization], isLoading: false, error: null, refresh: vi.fn() }}>
          <OrgMemberProvider value={{ members: [mockUser, mockUser2], isLoading: false, error: null, refresh: vi.fn() }}>
            <ChannelProvider value={{ channels, isLoading: false, error: null, refresh: vi.fn() }}>
              <DirectMessageProvider value={{ conversations, isLoading: false, error: null, refresh: vi.fn() }}>
                <ReadStateProvider value={{ counts, total: Object.values(counts).reduce((a, b) => a + b, 0), increment: vi.fn(), markRead: vi.fn(), applyRemoteRead: vi.fn(), refresh: vi.fn() }}>
                  <Routes>
                    <Route path="/org/:organizationId" element={<OrgHome />} />
                  </Routes>
                </ReadStateProvider>
              </DirectMessageProvider>
            </ChannelProvider>
          </OrgMemberProvider>
        </OrganizationProvider>
      </SessionProvider>
    </MemoryRouter>,
  );
}

describe('OrgHome', () => {
  it('renders the organization name', () => {
    renderOrgHome();
    expect(screen.getByRole('heading', { name: 'Acme' })).toBeInTheDocument();
  });

  it('lists channels and direct messages with unread counts', () => {
    renderOrgHome({
      channels: [
        { ...mockChannel, id: 'chan-1', name: 'general' },
        { ...mockChannel, id: 'chan-2', name: 'random' },
      ],
      conversations: [
        { id: 'dm-1', organization_id: 'org-1', member_ids: ['user-2'], created_at: '2026-01-01T00:00:00Z' },
      ],
      counts: { 'chan-1': 3, 'dm-1': 1 },
    });

    expect(screen.getByText('# general')).toBeInTheDocument();
    expect(screen.getByText('# random')).toBeInTheDocument();
    expect(screen.getByText('Bob')).toBeInTheDocument();
  });

  it('sorts conversations by unread count descending', () => {
    renderOrgHome({
      channels: [
        { ...mockChannel, id: 'chan-1', name: 'general' },
        { ...mockChannel, id: 'chan-2', name: 'random' },
      ],
      counts: { 'chan-1': 5, 'chan-2': 10 },
    });

    const links = screen.getAllByRole('link');
    expect(links[0]).toHaveTextContent('# random');
    expect(links[1]).toHaveTextContent('# general');
  });

  it('shows an empty state when there are no conversations', () => {
    renderOrgHome();
    expect(screen.getByText(/No conversations yet/i)).toBeInTheDocument();
  });
});
