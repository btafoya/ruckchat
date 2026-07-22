import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

const TYPING_TIMEOUT_MS = 3000;

export interface TypingState {
  typingUsers: Record<string, string[]>;
  addTypingUser: (conversationId: string, userId: string) => void;
  removeTypingUser: (conversationId: string, userId: string) => void;
}

export function useTyping(): TypingState {
  const [typingUsers, setTypingUsers] = useState<Record<string, string[]>>({});
  const timersRef = useRef<Record<string, ReturnType<typeof setTimeout>>>({});

  const removeTypingUser = useCallback((conversationId: string, userId: string) => {
    setTypingUsers((prev) => {
      const users = prev[conversationId] ?? [];
      const filtered = users.filter((id) => id !== userId);
      if (filtered.length === users.length) {
        return prev;
      }
      const next = { ...prev, [conversationId]: filtered };
      if (next[conversationId].length === 0) {
        delete next[conversationId];
      }
      return next;
    });
  }, []);

  const addTypingUser = useCallback(
    (conversationId: string, userId: string) => {
      setTypingUsers((prev) => {
        const users = prev[conversationId] ?? [];
        if (users.includes(userId)) {
          return prev;
        }
        return { ...prev, [conversationId]: [...users, userId] };
      });

      const key = `${conversationId}:${userId}`;
      const existing = timersRef.current[key];
      if (existing) {
        clearTimeout(existing);
      }
      timersRef.current[key] = setTimeout(() => {
        removeTypingUser(conversationId, userId);
        delete timersRef.current[key];
      }, TYPING_TIMEOUT_MS);
    },
    [removeTypingUser],
  );

  useEffect(() => {
    return () => {
      for (const timer of Object.values(timersRef.current)) {
        clearTimeout(timer);
      }
    };
  }, []);

  return useMemo(
    () => ({
      typingUsers,
      addTypingUser,
      removeTypingUser,
    }),
    [typingUsers, addTypingUser, removeTypingUser],
  );
}
