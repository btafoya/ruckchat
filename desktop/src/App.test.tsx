import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import App from './App';

const mockListOrganizations = vi.fn().mockResolvedValue({ items: [] });
const mockListChannels = vi.fn().mockResolvedValue({ items: [] });
const mockGetProfile = vi.fn().mockResolvedValue(null);
const mockLogin = vi.fn();
const mockLogout = vi.fn();
const mockRegister = vi.fn();

vi.mock('./api', async () => {
  const actual = await import('./api');
  return {
    ...actual,
    createApi: () => ({
      organizations: {
        list: mockListOrganizations,
        listChannels: mockListChannels,
      },
      auth: {
        getProfile: mockGetProfile,
        getRegistrationStatus: vi.fn().mockResolvedValue({ allow_registration: true }),
        login: mockLogin,
        logout: mockLogout,
        register: mockRegister,
      },
    }),
  };
});

const SETTINGS_KEY = 'ruckchat_settings';

describe('App', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders the first-run setup screen when no server URL is configured', async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Welcome to RuckChat/i })).toBeInTheDocument();
    });
  });

  it('renders the sign-in screen when the server URL is already configured', async () => {
    localStorage.setItem(
      SETTINGS_KEY,
      JSON.stringify({
        apiUrl: 'http://localhost:3000',
        notificationsEnabled: true,
        theme: 'system',
        sidebarCollapsed: false,
        serverUrlConfigured: true,
      }),
    );
    render(<App />);
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Sign in to RuckChat/i })).toBeInTheDocument();
    });
  });
});
