import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';

describe('useMessages offline retry', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('keeps failed sends in a pending state for retry', async () => {
    const postMessage = vi.fn().mockRejectedValue(new Error('network down'));

    vi.doMock('../api', () => ({
      createApi: () => ({
        channels: {
          postMessage,
          listMessages: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
          listReplies: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
        },
        directMessages: { postMessage: vi.fn() },
        files: { attachToMessage: vi.fn() },
      }),
    }));

    const { useMessages: useMessagesMocked } = await import('./useMessages');
    const { result } = renderHook(() => useMessagesMocked('token', 'channel', 'conv-1', 'user-1'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.sendMessage('hello');
    });

    const pending = result.current.messages.find((m: { id: string }) => m.id.startsWith('pending-'));
    expect(pending).toBeDefined();
    expect(pending?.content).toBe('hello');
    expect(result.current.error).toBe('network down');

    vi.doUnmock('../api');
  });
});

describe('useMessages appendMessage conversation scoping', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('drops a live-appended message belonging to a different conversation', async () => {
    vi.doMock('../api', () => ({
      createApi: () => ({
        channels: {
          listMessages: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
          listReplies: vi.fn().mockResolvedValue({ items: [], has_more_older: false, has_more_newer: false }),
          postMessage: vi.fn(),
        },
        directMessages: { postMessage: vi.fn() },
        files: { attachToMessage: vi.fn() },
      }),
    }));

    const { useMessages: useMessagesMocked } = await import('./useMessages');
    // The single shared WebSocket connection delivers events for every
    // conversation the user belongs to (server auto-subscribes per
    // organization/DM); this hook instance is bound to 'conv-1' only.
    const { result } = renderHook(() => useMessagesMocked('token', 'channel', 'conv-1', 'user-1'));

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    act(() => {
      result.current.appendMessage({
        id: 'msg-from-other-conversation',
        conversation_id: 'conv-2',
        conversation_type: 'channel',
        author_id: 'user-2',
        content: 'should not appear here',
        mentioned_user_ids: [],
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        deleted_at: null,
      });
    });

    expect(result.current.messages).toHaveLength(0);

    act(() => {
      result.current.appendMessage({
        id: 'msg-in-this-conversation',
        conversation_id: 'conv-1',
        conversation_type: 'channel',
        author_id: 'user-2',
        content: 'belongs here',
        mentioned_user_ids: [],
        created_at: '2026-01-01T00:00:01Z',
        updated_at: '2026-01-01T00:00:01Z',
        deleted_at: null,
      });
    });

    expect(result.current.messages).toHaveLength(1);
    expect(result.current.messages[0].id).toBe('msg-in-this-conversation');

    vi.doUnmock('../api');
  });
});
