import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { useWebSocket } from './useWebsocket';

class MockWebSocket {
  static instances: MockWebSocket[] = [];
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;
  readyState = 0;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: MessageEvent<string>) => void) | null = null;
  sent: string[] = [];

  constructor(_url: string) {
    MockWebSocket.instances.push(this);
  }

  set readyStateValue(value: number) {
    this.readyState = value;
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = 3;
    if (this.onclose) {
      this.onclose();
    }
  }

  triggerOpen() {
    this.readyState = 1;
    if (this.onopen) {
      this.onopen();
    }
  }

  triggerMessage(data: string) {
    if (this.onmessage) {
      this.onmessage({ data } as MessageEvent<string>);
    }
  }
}

describe('useWebSocket', () => {
  it('connects when a token is provided and dispatches events', async () => {
    const handler = vi.fn();
    vi.stubGlobal('WebSocket', MockWebSocket);
    vi.stubGlobal('location', { origin: 'http://localhost:3000' });
    MockWebSocket.instances = [];

    const { result } = renderHook(() => useWebSocket('token', handler));

    await act(async () => {
      await Promise.resolve();
    });

    await waitFor(
      () => {
        expect(MockWebSocket.instances.length).toBeGreaterThan(0);
      },
      { timeout: 3000 },
    );

    expect(MockWebSocket.instances[0]).toBeDefined();

    const socket = MockWebSocket.instances[0];

    act(() => {
      socket.triggerOpen();
    });

    await waitFor(() => {
      expect(result.current.status).toBe('open');
    });

    act(() => {
      socket.triggerMessage(
        JSON.stringify({
          type: 'message.created',
          id: '00000000-0000-0000-0000-000000000001',
          timestamp: '2026-01-01T00:00:00Z',
          payload: { type: 'message.created', message: { id: 'msg-1', content: 'hi' } },
        }),
      );
    });

    await waitFor(() => {
      expect(handler).toHaveBeenCalled();
    });

    vi.unstubAllGlobals();
  });
});
