import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Composer } from './Composer';
import {
  DirectMessageProvider,
  MessageProvider,
  PlatformProvider,
  RealtimeProvider,
  SessionProvider,
} from '../context';
import { mockPlatform } from '../test/platformMocks';
import type { MessagesState } from '../hooks/useMessages';
import type { Message } from '../api';

const mockSendMessage = vi.fn().mockResolvedValue({ id: 'msg-1' } as Message);
const mockSendWs = vi.fn().mockReturnValue(true);
const mockRecordUpload = vi.fn().mockResolvedValue({ id: 'file-1', file_name: 'notes.txt' });
const mockSearchMembers = vi.fn().mockImplementation(async (_token: string, _orgId: string, query: string) => {
  if (query.toLowerCase().includes('user')) {
    return [
      { id: 'user-2', email: 'user2@example.com', display_name: 'user-2', avatar_url: null, is_server_admin: false },
    ];
  }
  return [];
});

function clearMocks() {
  mockSendMessage.mockClear();
  mockSendWs.mockClear();
  mockRecordUpload.mockClear();
  mockSearchMembers.mockClear();
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
      organizations: {
        searchMembers: mockSearchMembers,
      },
    }),
  };
});

function emptyDoc() {
  return { type: 'doc', content: [{ type: 'paragraph', content: [] }] };
}

function docFromText(text: string) {
  return { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text }] }] };
}

function textFromDoc(doc: unknown): string {
  const paragraph = (doc as { content?: Array<{ content?: Array<{ text?: string }> }> })?.content?.[0];
  if (!paragraph?.content) {
    return '';
  }
  return paragraph.content.map((node) => node.text ?? '').join('');
}

let mentionSuggestion: { items?: (args: { query: string }) => Promise<unknown> } | null = null;

vi.mock('@tiptap/react', async () => {
  const React = await import('react');
  const { useState, useRef, useEffect, useLayoutEffect } = React;

  function createTextarea(
    editorRef: React.MutableRefObject<any>,
    setDocRef: React.MutableRefObject<(value: unknown) => void>,
    emitUpdate: () => void,
  ): HTMLTextAreaElement {
    const textarea = document.createElement('textarea');
    textarea.placeholder = 'Type a message...';
    const handleInput = (event: Event) => {
      const value = (event.target as HTMLTextAreaElement).value;
      const newDoc = docFromText(value);
      setDocRef.current(newDoc);
      if (editorRef.current) {
        editorRef.current.isEmpty = !value.trim();
        editorRef.current.getJSON = () => newDoc;
      }
      emitUpdate();
      const match = value.match(/@(\S*)$/);
      if (match && mentionSuggestion?.items) {
        void mentionSuggestion.items({ query: match[1] });
      }
    };
    textarea.addEventListener('input', handleInput);
    textarea.addEventListener('change', handleInput);
    return textarea;
  }

  return {
    useEditor: ({ content, onUpdate, editorProps }: any) => {
      const [doc, setDoc] = useState(() => content || emptyDoc());
      const setDocRef = useRef(setDoc);
      useEffect(() => {
        setDocRef.current = setDoc;
      });
      const listenersRef = useRef<{ update: Array<() => void> }>({ update: [] });
      const emitUpdate = () => {
        listenersRef.current.update.forEach((cb) => cb());
      };

      const editorRef = useRef<any>(null);
      const textareaRef = useRef<HTMLTextAreaElement | null>(null);
      if (!textareaRef.current) {
        textareaRef.current = createTextarea(editorRef, setDocRef, emitUpdate);
        Object.assign(textareaRef.current, editorProps?.attributes ?? {});
      }

      if (!editorRef.current) {
        editorRef.current = {
          isEmpty: true,
          getJSON: () => doc,
          commands: {
            clearContent: () => {
              const newDoc = emptyDoc();
              setDocRef.current(newDoc);
              if (textareaRef.current) {
                textareaRef.current.value = '';
              }
              editorRef.current.isEmpty = true;
              editorRef.current.getJSON = () => newDoc;
              emitUpdate();
            },
            setContent: (c: unknown) => {
              setDocRef.current(c);
              const text = textFromDoc(c);
              if (textareaRef.current) {
                textareaRef.current.value = text;
              }
              editorRef.current.isEmpty = !text.trim();
              editorRef.current.getJSON = () => c;
              emitUpdate();
            },
          },
          setEditable: vi.fn(),
          on: (event: string, cb: () => void) => {
            if (event === 'update') {
              listenersRef.current.update.push(cb);
            }
          },
          off: (event: string, cb: () => void) => {
            if (event === 'update') {
              listenersRef.current.update = listenersRef.current.update.filter((l) => l !== cb);
            }
          },
          view: {
            dom: textareaRef.current,
            domAtPos: () => ({ node: textareaRef.current, offset: 0 }),
          },
        };
      }
      editorRef.current.isEmpty = !textFromDoc(doc).trim();
      editorRef.current.getJSON = () => doc;
      return editorRef.current;
    },
    EditorContent: ({ editor }: any) => {
      const wrapperRef = useRef<HTMLDivElement>(null);
      useLayoutEffect(() => {
        const el = editor?.view?.dom;
        if (wrapperRef.current && el && el.parentNode !== wrapperRef.current) {
          wrapperRef.current.appendChild(el);
          el.value = textFromDoc(editor?.getJSON());
        }
      });
      return <div ref={wrapperRef} data-testid="composer-editor" />;
    },
    ReactRenderer: class {
      element = document.createElement('div');
      updateProps() {}
      destroy() {}
      ref = null;
    },
  };
});

vi.mock('@tiptap/starter-kit', () => ({
  default: class StarterKit {
    static configure() {
      return new StarterKit();
    }
  },
}));
vi.mock('@tiptap/extension-mention', () => ({
  default: class Mention {
    static configure(config: { suggestion?: typeof mentionSuggestion }) {
      if (config?.suggestion) {
        mentionSuggestion = config.suggestion;
      }
      return new Mention();
    }
  },
}));
vi.mock('@tiptap/extension-placeholder', () => ({
  default: class Placeholder {
    static configure() {
      return new Placeholder();
    }
  },
}));
vi.mock('@tiptap/suggestion', () => ({ default: {} }));
vi.mock('tippy.js', () => ({
  default: () => ({ setProps: vi.fn(), destroy: vi.fn() }),
}));

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
  user: { id: 'user-1', email: 'user@example.com', display_name: 'User', avatar_url: null, is_server_admin: false },
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
          <RealtimeProvider value={{ status: 'open', send: mockSendWs }}>
            <PlatformProvider platform={mockPlatform}>{children}</PlatformProvider>
          </RealtimeProvider>
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
    await act(async () => {
      fireEvent.change(input, { target: { value: 'hello world' } });
    });
    fireEvent.keyDown(input, { key: 'Enter', code: 'Enter' });

    await waitFor(() => {
      expect(mockSendMessage).toHaveBeenCalledTimes(1);
    });
    const call = mockSendMessage.mock.calls[0];
    expect(JSON.parse(call[0] as string)).toEqual(docFromText('hello world'));
    expect(call[1]).toBeUndefined();
    expect(call[2]).toEqual([]);
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

  it('shows mention autocomplete when @ is typed', async () => {
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
      fireEvent.change(input, { target: { value: 'hello @user' } });
    });
    await waitFor(() => {
      expect(mockSearchMembers).toHaveBeenCalledWith('token', 'org-1', 'user');
    });
  });

  it('toggles a simple preview', async () => {
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
      fireEvent.change(input, { target: { value: 'hello world' } });
    });
    fireEvent.click(screen.getByRole('button', { name: /Preview/i }));
    await waitFor(() => {
      expect(screen.getByText('hello world')).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('button', { name: /Edit/i }));
    expect(screen.getByPlaceholderText(/Type a message/i)).toBeInTheDocument();
  });
});
