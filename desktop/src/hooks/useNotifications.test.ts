import { renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

const mockMessage = {
  id: 'msg-1',
  conversation_id: 'conv-1',
  conversation_type: 'channel' as const,
  author_id: 'user-1',
  content: 'hello',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  deleted_at: null,
};

vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: vi.fn().mockResolvedValue(false),
  requestPermission: vi.fn().mockResolvedValue('granted'),
  sendNotification: vi.fn(),
}));

describe('useNotifications', () => {
  it('notifies on direct messages when enabled', async () => {
    const { useNotifications } = await import('./useNotifications');
    const { sendNotification } = await import('@tauri-apps/plugin-notification');

    const { result } = renderHook(() => useNotifications({ userId: 'user-2', enabled: true }));

    await waitFor(() => {
      expect(result.current).toBeDefined();
    });

    await result.current.maybeNotify({
      type: 'message.created',
      message: {
        ...mockMessage,
        conversation_type: 'direct_message',
        author_id: 'user-3',
      },
    });

    expect(sendNotification).toHaveBeenCalled();
  });
});
