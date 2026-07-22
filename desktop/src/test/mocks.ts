import type { Channel, Organization, User } from '../api';
import type { Session } from '../hooks/useSession';

export const mockUser: User = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'alice@example.com',
  display_name: 'Alice',
  avatar_url: null,
};

export const mockOrganization: Organization = {
  id: '00000000-0000-0000-0000-000000000010',
  name: 'Acme',
  slug: 'acme',
  owner_id: mockUser.id,
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

export const mockChannel: Channel = {
  id: '00000000-0000-0000-0000-000000000020',
  organization_id: mockOrganization.id,
  name: 'general',
  topic: 'General discussion',
  purpose: null,
  is_private: false,
  created_by: mockUser.id,
  created_at: '2026-01-01T00:00:00Z',
  archived_at: null,
};

export const mockSession: Session = {
  token: 'test-token',
  user: mockUser,
};
