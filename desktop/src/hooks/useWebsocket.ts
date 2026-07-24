import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { ClientMessage, EventEnvelope, ServerEvent } from '../api/events';

export type EventHandler = (event: ServerEvent) => void;

export type ConnectionStatus = 'connecting' | 'open' | 'closed' | 'error';

const INITIAL_RECONNECT_DELAY_MS = 500;
const MAX_RECONNECT_DELAY_MS = 30000;

function buildWebSocketUrl(baseUrl: string): string {
  const url = new URL(baseUrl);
  const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${url.host}/websocket`;
}

export interface WebSocketState {
  status: ConnectionStatus;
  send: (message: ClientMessage) => boolean;
}

export interface UseWebSocketOptions {
  apiUrl?: string;
}

export function useWebSocket(
  token: string | undefined,
  onEvent: EventHandler,
  options: UseWebSocketOptions = {},
): WebSocketState {
  const [status, setStatus] = useState<ConnectionStatus>('closed');
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectDelayRef = useRef(INITIAL_RECONNECT_DELAY_MS);
  const onEventRef = useRef(onEvent);

  useEffect(() => {
    onEventRef.current = onEvent;
  }, [onEvent]);

  const connect = useCallback(() => {
    if (!token || socketRef.current?.readyState === WebSocket.CONNECTING) {
      return;
    }

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    setStatus('connecting');
    const baseUrl = options.apiUrl?.trim() || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost:3000');
    const url = buildWebSocketUrl(baseUrl);
    const socket = new WebSocket(url);
    socketRef.current = socket;

    socket.onopen = () => {
      setStatus('open');
      reconnectDelayRef.current = INITIAL_RECONNECT_DELAY_MS;
    };

    socket.onmessage = (event: MessageEvent<string>) => {
      try {
        const envelope = JSON.parse(event.data) as EventEnvelope;
        if (envelope.payload) {
          onEventRef.current(envelope.payload);
        }
      } catch (err) {
        console.warn('Failed to parse WebSocket message', err);
      }
    };

    socket.onclose = () => {
      socketRef.current = null;
      setStatus('closed');
      reconnectTimeoutRef.current = setTimeout(() => {
        reconnectDelayRef.current = Math.min(
          reconnectDelayRef.current * 2,
          MAX_RECONNECT_DELAY_MS,
        );
        connect();
      }, reconnectDelayRef.current);
    };

    socket.onerror = () => {
      setStatus('error');
    };
  }, [token, options.apiUrl]);

  const send = useCallback((message: ClientMessage): boolean => {
    const socket = socketRef.current;
    if (socket?.readyState !== WebSocket.OPEN) {
      return false;
    }
    socket.send(JSON.stringify(message));
    return true;
  }, []);

  useEffect(() => {
    if (!token) {
      setStatus('closed');
      if (socketRef.current) {
        socketRef.current.close();
        socketRef.current = null;
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
      return;
    }

    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
      if (socketRef.current) {
        socketRef.current.close();
        socketRef.current = null;
      }
    };
  }, [token, connect, options.apiUrl]);

  return useMemo(
    () => ({
      status,
      send,
    }),
    [status, send],
  );
}
