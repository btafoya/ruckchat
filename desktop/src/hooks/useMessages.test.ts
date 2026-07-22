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
          listMessages: vi.fn().mockResolvedValue([]),
          listReplies: vi.fn().mockResolvedValue([]),
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
