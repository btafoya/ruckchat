import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { Shell } from './Shell';
import { SessionProvider } from '../context';
import { mockSession } from '../test/mocks';
import { AuthScreen } from './AuthScreen';

const mockListOrganizations = vi.fn().mockResolvedValue([]);
const mockListChannels = vi.fn().mockResolvedValue([]);

vi.mock('../api', async () => {
  const actual = await import('../api');
  return {
    ...actual,
    createApi: () => ({
      organizations: {
        list: mockListOrganizations,
        listChannels: mockListChannels,
      },
      auth: {
        getProfile: vi.fn().mockResolvedValue(mockSession.user),
        login: vi.fn(),
        logout: vi.fn().mockResolvedValue(undefined),
      },
    }),
  };
});

function renderWithSession(session: import('../hooks/useSession').Session | null = mockSession, initialEntries = ['/']) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <SessionProvider
        value={{
          session,
          isLoading: false,
          error: null,
          login: vi.fn(),
          register: vi.fn(),
          logout: vi.fn(),
        }}
      >
        <Routes>
          <Route path="/login" element={<AuthScreen />} />
          <Route path="/*" element={<Shell />} />
        </Routes>
      </SessionProvider>
    </MemoryRouter>,
  );
}

describe('Shell', () => {
  it('redirects unauthenticated users to login', async () => {
    renderWithSession(null);
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Sign in to RuckChat/i })).toBeInTheDocument();
    });
  });

  it('shows the sidebar for an authenticated user', async () => {
    renderWithSession(mockSession);
    await waitFor(() => {
      expect(screen.getByRole('navigation', { name: /Organizations/i })).toBeInTheDocument();
    });
    expect(screen.getByText(mockSession.user.display_name)).toBeInTheDocument();
  });
});
