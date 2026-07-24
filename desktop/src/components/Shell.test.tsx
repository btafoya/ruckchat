import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { Shell } from './Shell';
import {
  ChannelProvider,
  DirectMessageProvider,
  MessageProvider,
  OrganizationProvider,
  PresenceProvider,
  RealtimeProvider,
  SessionProvider,
  TypingProvider,
} from '../context';
import { mockSession } from '../test/mocks';
import { AuthScreen } from './AuthScreen';
import {
  useMessages,
  useOrganizations,
  useChannels,
  usePresence,
  useTyping,
  useDirectMessages,
} from '../hooks';

const mockListOrganizations = vi.fn().mockResolvedValue([]);
const mockListChannels = vi.fn().mockResolvedValue([]);
const mockListMessages = vi.fn().mockResolvedValue([]);

vi.mock('../api', async () => {
  const actual = await import('../api');
  return {
    ...actual,
    createApi: () => ({
      organizations: {
        list: mockListOrganizations,
        listChannels: mockListChannels,
      },
      channels: {
        listMessages: mockListMessages,
        listReplies: vi.fn().mockResolvedValue([]),
        postMessage: vi.fn().mockResolvedValue({}),
      },
      directMessages: {
        list: vi.fn().mockResolvedValue([]),
        listMessages: vi.fn().mockResolvedValue([]),
        postMessage: vi.fn().mockResolvedValue({}),
      },
      reactions: {
        add: vi.fn().mockResolvedValue({}),
        remove: vi.fn().mockResolvedValue(undefined),
      },
      files: {
        recordUpload: vi.fn().mockResolvedValue({ id: 'file-1', file_name: 'test.txt' }),
        attachToMessage: vi.fn().mockResolvedValue(undefined),
      },
      auth: {
        getProfile: vi.fn().mockResolvedValue(mockSession.user),
        getRegistrationStatus: vi.fn().mockResolvedValue({ allow_registration: true }),
        login: vi.fn(),
        logout: vi.fn().mockResolvedValue(undefined),
      },
    }),
  };
});

vi.mock('../hooks/useWebsocket', () => ({
  useWebSocket: () => ({
    status: 'closed' as const,
    send: vi.fn().mockReturnValue(true),
  }),
}));

function Wrapper({ session, children }: { session: import('../hooks/useSession').Session | null; children: React.ReactNode }) {
  const organizationsState = useOrganizations(session?.token);
  const channelsState = useChannels(session?.token, undefined);
  const directMessagesState = useDirectMessages(session?.token, undefined);
  const messagesState = useMessages(session?.token, undefined, undefined, session?.user.id);
  const presenceState = usePresence();
  const typingState = useTyping();

  return (
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
      <OrganizationProvider value={organizationsState}>
        <ChannelProvider value={channelsState}>
          <DirectMessageProvider value={directMessagesState}>
            <MessageProvider value={messagesState}>
              <PresenceProvider value={presenceState}>
                <TypingProvider value={typingState}>
                  <RealtimeProvider value={{ status: 'closed', send: vi.fn().mockReturnValue(true) }}>
                    {children}
                  </RealtimeProvider>
                </TypingProvider>
              </PresenceProvider>
            </MessageProvider>
          </DirectMessageProvider>
        </ChannelProvider>
      </OrganizationProvider>
    </SessionProvider>
  );
}

function renderWithSession(
  session: import('../hooks/useSession').Session | null = mockSession,
  initialEntries = ['/'],
) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <Wrapper session={session}>
        <Routes>
          <Route path="/login" element={<AuthScreen />} />
          <Route path="/*" element={<Shell />} />
        </Routes>
      </Wrapper>
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

