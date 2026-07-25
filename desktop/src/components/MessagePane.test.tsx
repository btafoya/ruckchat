import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { MessagePane } from './MessagePane';
import {
  ChannelProvider,
  DirectMessageProvider,
  MessageProvider,
  OrgMemberProvider,
  OrganizationProvider,
  PlatformProvider,
  PresenceProvider,
  RealtimeProvider,
  SessionProvider,
  SettingsProvider,
  TypingProvider,
} from '../context';
import { mockPlatform } from '../test/platformMocks';
import type { MessagesState } from '../hooks/useMessages';
import type { Message, Channel, Organization, DirectMessageConversation } from '../api';

const mockAddReaction = vi.fn().mockResolvedValue({
  message_id: 'msg-1',
  user_id: 'user-1',
  emoji: '👍',
  created_at: '2026-01-01T00:00:00Z',
});
const mockRemoveReaction = vi.fn().mockResolvedValue(undefined);

vi.mock('../api', async () => {
  const actual = await import('../api');
  return {
    ...actual,
    createApi: () => ({
      channels: {
        listMessages: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
        listReplies: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
        postMessage: vi.fn().mockResolvedValue({}),
        markRead: vi.fn().mockResolvedValue(undefined),
      },
      directMessages: {
        list: vi.fn().mockResolvedValue([]),
        listMessages: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
        postMessage: vi.fn().mockResolvedValue({}),
        markRead: vi.fn().mockResolvedValue(undefined),
      },
      reactions: {
        add: mockAddReaction,
        remove: mockRemoveReaction,
      },
      files: {
        recordUpload: vi.fn().mockResolvedValue({ id: 'file-1', file_name: 'test.txt' }),
        attachToMessage: vi.fn().mockResolvedValue(undefined),
      },
    }),
  };
});

const mockOrganization: Organization = {
  id: 'org-1',
  name: 'Acme',
  slug: 'acme',
  owner_id: 'user-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

const mockChannel: Channel = {
  id: 'chan-1',
  organization_id: 'org-1',
  name: 'general',
  topic: 'General discussion',
  purpose: null,
  is_private: false,
  created_by: 'user-1',
  created_at: '2026-01-01T00:00:00Z',
  archived_at: null,
};

const mockMessage: Message = {
  id: 'msg-1',
  conversation_id: 'chan-1',
  conversation_type: 'channel',
  author_id: 'user-1',
  content: 'Hello everyone',
  mentioned_user_ids: [],
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  deleted_at: null,
};

const mockMessageState: MessagesState = {
  messages: [mockMessage],
  isLoading: false,
  isLoadingOlder: false,
  isLoadingNewer: false,
  error: null,
  hasMoreOlder: true,
  hasMoreNewer: false,
  unreadIds: new Set(),
  lastAppendedId: null,
  refresh: vi.fn(),
  loadOlder: vi.fn().mockResolvedValue(undefined),
  loadNewer: vi.fn().mockResolvedValue(undefined),
  jumpToMessage: vi.fn().mockResolvedValue(undefined),
  sendMessage: vi.fn().mockResolvedValue(undefined),
  retryMessage: vi.fn().mockResolvedValue(undefined),
  loadThreadReplies: vi.fn().mockResolvedValue(undefined),
  loadOlderReplies: vi.fn().mockResolvedValue(undefined),
  loadNewerReplies: vi.fn().mockResolvedValue(undefined),
  threadReplies: [],
  threadRepliesLoading: false,
  threadHasMoreOlder: true,
  threadHasMoreNewer: false,
  reactions: {
    'msg-1': [
      {
        message_id: 'msg-1',
        user_id: 'user-2',
        emoji: '👍',
        created_at: '2026-01-01T00:00:00Z',
      },
    ],
  },
  addReaction: vi.fn(),
  removeReaction: vi.fn(),
  appendMessage: vi.fn(),
  updateMessage: vi.fn(),
  removeMessage: vi.fn(),
  markRead: vi.fn(),
  editingMessage: null,
  startEdit: vi.fn(),
  cancelEdit: vi.fn(),
  saveEdit: vi.fn(),
};

const mockSession = {
  token: 'token',
  user: { id: 'user-1', email: 'user@example.com', display_name: 'User', avatar_url: null, is_server_admin: false },
};

function renderPane(
  initialEntries = ['/org/org-1/channel/chan-1'],
  messageState: MessagesState = mockMessageState,
) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <SettingsProvider
        value={{
          apiUrl: 'http://localhost:3000',
          notificationsEnabled: false,
          theme: 'system',
          isLoading: false,
          resolvedTheme: 'dark',
          setApiUrl: vi.fn(),
          setNotificationsEnabled: vi.fn(),
          setTheme: vi.fn(),
          reset: vi.fn(),
        }}
      >
      <SessionProvider
        value={{
          session: mockSession,
          isLoading: false,
          error: null,
          login: vi.fn(),
          register: vi.fn(),
          logout: vi.fn(),
        }}
      >
        <OrganizationProvider
          value={{
            organizations: [mockOrganization],
            isLoading: false,
            error: null,
            refresh: vi.fn(),
          }}
        >
          <OrgMemberProvider
            value={{
              members: [],
              isLoading: false,
              error: null,
              refresh: vi.fn(),
            }}
          >
            <ChannelProvider
              value={{
                channels: [mockChannel],
                isLoading: false,
                error: null,
                refresh: vi.fn(),
              }}
            >
              <DirectMessageProvider
                value={{
                  conversations: [] as DirectMessageConversation[],
                  isLoading: false,
                  error: null,
                  refresh: vi.fn(),
                }}
              >
                <MessageProvider value={messageState}>
                  <PresenceProvider value={{ presence: {}, setUserPresence: vi.fn() }}>
                    <TypingProvider
                      value={{
                        typingUsers: { 'chan-1': ['user-2'] },
                        addTypingUser: vi.fn(),
                        removeTypingUser: vi.fn(),
                      }}
                    >
                      <RealtimeProvider value={{ status: 'open', send: vi.fn() }}>
                        <PlatformProvider platform={mockPlatform}>
                          <Routes>
                            <Route path="/org/:organizationId/channel/:channelId" element={<MessagePane />} />
                            <Route
                              path="/org/:organizationId/channel/:channelId/thread/:messageId"
                              element={<MessagePane />}
                            />
                            <Route path="/org/:organizationId/dm/:dmId" element={<MessagePane />} />
                          </Routes>
                        </PlatformProvider>
                      </RealtimeProvider>
                    </TypingProvider>
                  </PresenceProvider>
                </MessageProvider>
              </DirectMessageProvider>
            </ChannelProvider>
          </OrgMemberProvider>
        </OrganizationProvider>
      </SessionProvider>
      </SettingsProvider>
    </MemoryRouter>,
  );
}

describe('MessagePane', () => {
  it('renders the channel title and messages', () => {
    renderPane();
    expect(screen.getByText(/# general/i)).toBeInTheDocument();
    expect(screen.getByText(/Hello everyone/i)).toBeInTheDocument();
  });

  it('shows an unread dot for an unread message from another user', () => {
    const otherAuthorMessage: Message = { ...mockMessage, id: 'msg-2', author_id: 'user-2' };
    const state: MessagesState = {
      ...mockMessageState,
      messages: [otherAuthorMessage],
      unreadIds: new Set(['msg-2']),
    };
    renderPane(['/org/org-1/channel/chan-1'], state);
    expect(screen.getByLabelText('Unread message')).toBeInTheDocument();
  });

  it("does not show an unread dot for the caller's own message", () => {
    // mockMessage is authored by 'user-1', the same id as mockSession.user.id.
    const state: MessagesState = { ...mockMessageState, unreadIds: new Set(['msg-1']) };
    renderPane(['/org/org-1/channel/chan-1'], state);
    expect(screen.queryByLabelText('Unread message')).not.toBeInTheDocument();
  });

  it('renders existing reactions', () => {
    renderPane();
    expect(screen.getByRole('button', { name: '👍' })).toBeInTheDocument();
  });

  it('toggles the current user reaction when a reaction chip is clicked', async () => {
    renderPane();
    fireEvent.click(screen.getByRole('button', { name: '👍' }));
    await waitFor(() => {
      expect(mockAddReaction).toHaveBeenCalledWith('token', 'msg-1', '👍');
    });
  });

  it('renders a typing indicator for other users', () => {
    renderPane();
    expect(screen.getByText(/user-2 is typing/i)).toBeInTheDocument();
  });

  it('renders the thread pane on a thread route', () => {
    renderPane(['/org/org-1/channel/chan-1/thread/msg-1']);
    expect(screen.getByText('Thread')).toBeInTheDocument();
  });
});
