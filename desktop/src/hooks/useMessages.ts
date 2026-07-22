import { useCallback, useEffect, useMemo, useState } from 'react';
import { createApi } from '../api';
import type { Message } from '../api';

export interface MessagesState {
  messages: Message[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  appendMessage: (message: Message) => void;
  updateMessage: (message: Message) => void;
  removeMessage: (messageId: string) => void;
}

export function useMessages(
  token: string | undefined,
  conversationType: 'channel' | 'direct_message' | undefined,
  conversationId: string | undefined,
): MessagesState {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useMemo(() => createApi(), []);

  const refresh = useCallback(async () => {
    if (!token || !conversationType || !conversationId) {
      setMessages([]);
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const items =
        conversationType === 'channel'
          ? await api.channels.listMessages(token, conversationId)
          : await api.directMessages.listMessages(token, conversationId);
      setMessages(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load messages');
    } finally {
      setIsLoading(false);
    }
  }, [api, token, conversationType, conversationId]);

  const appendMessage = useCallback((message: Message) => {
    setMessages((prev) => {
      if (prev.some((m) => m.id === message.id)) {
        return prev;
      }
      return [...prev, message];
    });
  }, []);

  const updateMessage = useCallback((message: Message) => {
    setMessages((prev) =>
      prev.map((m) => {
        if (m.id !== message.id) {
          return m;
        }
        return message;
      }),
    );
  }, []);

  const removeMessage = useCallback((messageId: string) => {
    setMessages((prev) => prev.filter((m) => m.id !== messageId));
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return useMemo(
    () => ({
      messages,
      isLoading,
      error,
      refresh,
      appendMessage,
      updateMessage,
      removeMessage,
    }),
    [messages, isLoading, error, refresh, appendMessage, updateMessage, removeMessage],
  );
}
