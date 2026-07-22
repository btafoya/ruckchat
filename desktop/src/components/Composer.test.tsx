import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Composer } from './Composer';
import {
  DirectMessageProvider,
  MessageProvider,
  RealtimeProvider,
  SessionProvider,
} from '../context';
import type { MessagesState } from '../hooks/useMessages';
import type { Message } from '../api';

const mockSendMessage = vi.fn().mockResolvedValue({ id: 'msg-1' } as Message);
const mockSendWs = vi.fn().mockReturnValue(true);
const mockRecordUpload = vi.fn().mockResolvedValue({ id: 'file-1', file_name: 'notes.txt' });

function clearMocks() {
  mockSendMessage.mockClear();
  mockSendWs.mockClear();
  mockRecordUpload.mockClear();
}

vi.mock('../api', async () => {
  const actual = await import('../api');
  return {
    ...actual,
    createApi: () => ({
      files: {
        recordUpload: mockRecordUpload,
        attachToMessage: vi.fn().mockResolvedValue(undefined),
      },
    }),
  };
});

const mockMessageState: MessagesState = {
  messages: [],
  isLoading: false,
  isLoadingMore: false,
  error: null,
  hasMore: false,
  refresh: vi.fn(),
  loadMore: vi.fn(),
  sendMessage: mockSendMessage,
  retryMessage: vi.fn(),
  loadThreadReplies: vi.fn(),
  threadReplies: [],
  threadRepliesLoading: false,
  reactions: {},
  addReaction: vi.fn(),
  removeReaction: vi.fn(),
  appendMessage: vi.fn(),
  updateMessage: vi.fn(),
  removeMessage: vi.fn(),
};

const mockSession = {
  token: 'token',
  user: { id: 'user-1', email: 'user@example.com', display_name: 'User', avatar_url: null },
};

function Wrapper({ children }: { children: React.ReactNode }) {
  return (
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
      <DirectMessageProvider
        value={{
          conversations: [
            {
              id: 'dm-1',
              organization_id: 'org-1',
              member_ids: ['user-1', 'user-2'],
              created_at: '2026-01-01T00:00:00Z',
            },
          ],
          isLoading: false,
          error: null,
          refresh: vi.fn(),
        }}
      >
        <MessageProvider value={mockMessageState}>
          <RealtimeProvider value={{ status: 'open', send: mockSendWs }}>{children}</RealtimeProvider>
        </MessageProvider>
      </DirectMessageProvider>
    </SessionProvider>
  );
}

describe('Composer', () => {
  beforeEach(() => {
    clearMocks();
  });

  it('renders the message input and send button', () => {
    render(
      <Wrapper>
        <Composer
          conversationType="channel"
          conversationId="chan-1"
          organizationId="org-1"
        />
      </Wrapper>,
    );
    expect(screen.getByPlaceholderText(/Type a message/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Send/i })).toBeInTheDocument();
  });

  it('sends a message on Enter and clears the input', async () => {
    render(
      <Wrapper>
        <Composer
          conversationType="channel"
          conversationId="chan-1"
          organizationId="org-1"
        />
      </Wrapper>,
    );
    const input = screen.getByPlaceholderText(/Type a message/i);
    fireEvent.change(input, { target: { value: 'hello world' } });
    fireEvent.keyDown(input, { key: 'Enter', code: 'Enter' });

    await waitFor(() => {
      expect(mockSendMessage).toHaveBeenCalledWith('hello world', undefined, []);
    });
    expect(input).toHaveValue('');
  });

  it('does not send on Shift+Enter', () => {
    render(
      <Wrapper>
        <Composer
          conversationType="channel"
          conversationId="chan-1"
          organizationId="org-1"
        />
      </Wrapper>,
    );
    const input = screen.getByPlaceholderText(/Type a message/i);
    fireEvent.change(input, { target: { value: 'multi\nline' } });
    fireEvent.keyDown(input, { key: 'Enter', shiftKey: true });
    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('sends a typing WebSocket message while composing', async () => {
    render(
      <Wrapper>
        <Composer
          conversationType="channel"
          conversationId="chan-1"
          organizationId="org-1"
        />
      </Wrapper>,
    );
    const input = screen.getByPlaceholderText(/Type a message/i);
    await act(async () => {
      fireEvent.change(input, { target: { value: 'h' } });
    });
    await waitFor(() => {
      expect(mockSendWs).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'typing',
          conversation_id: 'chan-1',
          conversation_type: 'channel',
        }),
      );
    });
  });

  it('shows mention autocomplete when @ is typed', () => {
    render(
      <Wrapper>
        <Composer
          conversationType="channel"
          conversationId="chan-1"
          organizationId="org-1"
        />
      </Wrapper>,
    );
    const input = screen.getByPlaceholderText(/Type a message/i);
    fireEvent.change(input, { target: { value: 'hello @user' } });
    expect(screen.getByText('@user-2')).toBeInTheDocument();
  });

  it('toggles a simple markdown preview', () => {
    render(
      <Wrapper>
        <Composer
          conversationType="channel"
          conversationId="chan-1"
          organizationId="org-1"
        />
      </Wrapper>,
    );
    const input = screen.getByPlaceholderText(/Type a message/i);
    fireEvent.change(input, { target: { value: '**bold**' } });
    fireEvent.click(screen.getByRole('button', { name: /Preview/i }));
    expect(screen.getByText('**bold**')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /Edit/i }));
    expect(screen.getByDisplayValue('**bold**')).toBeInTheDocument();
  });
});
