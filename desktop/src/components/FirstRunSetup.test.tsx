import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { FirstRunSetup } from './FirstRunSetup';
import { SettingsProvider } from '../context';
import { ApiError } from '../api/error';
import type { SettingsState } from '../hooks/useSettings';

const mockRequest = vi.fn();

vi.mock('../api/client', () => ({
  ApiClient: class MockApiClient {
    constructor(public readonly baseUrl: string) {}
    request<T>(path: string, options?: unknown): Promise<T> {
      return mockRequest(path, options) as Promise<T>;
    }
  },
}));

function buildSettingsState(overrides: Partial<SettingsState> = {}): SettingsState {
  const setApiUrl = vi.fn();
  const setTheme = vi.fn();
  const setNotificationsEnabled = vi.fn();
  const setServerUrlConfigured = vi.fn();
  return {
    apiUrl: 'http://localhost:3000',
    notificationsEnabled: true,
    theme: 'system',
    sidebarCollapsed: false,
    serverUrlConfigured: false,
    isLoading: false,
    resolvedTheme: 'dark',
    setApiUrl,
    setNotificationsEnabled,
    setTheme,
    setSidebarCollapsed: vi.fn(),
    setServerUrlConfigured,
    reset: vi.fn(),
    ...overrides,
  };
}

function renderFirstRunSetup(state = buildSettingsState()) {
  return render(
    <SettingsProvider value={state}>
      <FirstRunSetup />
    </SettingsProvider>,
  );
}

describe('FirstRunSetup', () => {
  beforeEach(() => {
    mockRequest.mockReset();
  });

  it('starts with an empty server URL field', () => {
    renderFirstRunSetup();
    expect(screen.getByLabelText(/Server URL/i)).toHaveValue('');
  });

  it('disables Connect when the URL is empty', () => {
    renderFirstRunSetup();
    expect(screen.getByRole('button', { name: /Connect/i })).toBeDisabled();
  });

  it('shows a validation error for an unsupported protocol', async () => {
    const user = userEvent.setup();
    renderFirstRunSetup();
    const input = screen.getByLabelText(/Server URL/i);
    await user.type(input, 'ftp://example.com');
    await user.click(screen.getByRole('button', { name: /Connect/i }));
    expect(
      screen.getByText(/URL must start with http:\/\/ or https:\/\//i),
    ).toBeInTheDocument();
  });

  it('saves settings when the server is reachable', async () => {
    const user = userEvent.setup();
    const state = buildSettingsState();
    mockRequest.mockResolvedValue({ allow_registration: true });
    renderFirstRunSetup(state);

    const input = screen.getByLabelText(/Server URL/i);
    await user.type(input, 'https://ruckchat.example.com');
    await user.click(screen.getByRole('button', { name: /Connect/i }));

    await waitFor(() => {
      expect(state.setApiUrl).toHaveBeenCalledWith('https://ruckchat.example.com');
      expect(state.setServerUrlConfigured).toHaveBeenCalledWith(true);
    });
    expect(mockRequest).toHaveBeenCalledTimes(1);
    expect(mockRequest.mock.calls[0][0]).toBe('/auth/registration-status');
  });

  it('reports a readable error for HTTP failures', async () => {
    const user = userEvent.setup();
    mockRequest.mockRejectedValue(new ApiError(503, { code: 'unavailable', error: 'temporarily down' }));
    renderFirstRunSetup();

    const input = screen.getByLabelText(/Server URL/i);
    await user.type(input, 'https://ruckchat.example.com');
    await user.click(screen.getByRole('button', { name: /Connect/i }));

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent(
        'Server responded with HTTP 503: temporarily down.',
      );
    });
  });

  it('reports a readable error for network failures', async () => {
    const user = userEvent.setup();
    mockRequest.mockRejectedValue(new TypeError('Failed to fetch'));
    renderFirstRunSetup();

    const input = screen.getByLabelText(/Server URL/i);
    await user.type(input, 'https://ruckchat.example.com');
    await user.click(screen.getByRole('button', { name: /Connect/i }));

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent(
        'Network error while contacting the server: Failed to fetch',
      );
    });
  });

  it('falls back to a generic message for unexpected thrown values', async () => {
    const user = userEvent.setup();
    mockRequest.mockImplementation(() => {
      throw 'unexpected';
    });
    renderFirstRunSetup();

    const input = screen.getByLabelText(/Server URL/i);
    await user.type(input, 'https://ruckchat.example.com');
    await user.click(screen.getByRole('button', { name: /Connect/i }));

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent(
        'Could not reach a RuckChat server at this URL.',
      );
    });
  });

  it('rejects a response missing the expected field', async () => {
    const user = userEvent.setup();
    mockRequest.mockResolvedValue({});
    renderFirstRunSetup();

    const input = screen.getByLabelText(/Server URL/i);
    await user.type(input, 'https://ruckchat.example.com');
    await user.click(screen.getByRole('button', { name: /Connect/i }));

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent(
        /Server did not return a valid RuckChat response/i,
      );
    });
  });
});
