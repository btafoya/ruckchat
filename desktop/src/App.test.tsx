import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
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
        login: mockLogin,
        logout: mockLogout,
        register: mockRegister,
      },
    }),
  };
});

describe('App', () => {
  it('renders the sign-in screen when not authenticated', async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Sign in to RuckChat/i })).toBeInTheDocument();
    });
  });
});
