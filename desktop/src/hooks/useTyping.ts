import { useCallback, useMemo, useState } from 'react';

export interface TypingState {
  typingUsers: Record<string, string[]>;
  addTypingUser: (conversationId: string, userId: string) => void;
  removeTypingUser: (conversationId: string, userId: string) => void;
}

export function useTyping(): TypingState {
  const [typingUsers, setTypingUsers] = useState<Record<string, string[]>>({});

  const addTypingUser = useCallback((conversationId: string, userId: string) => {
    setTypingUsers((prev) => {
      const users = prev[conversationId] ?? [];
      if (users.includes(userId)) {
        return prev;
      }
      return { ...prev, [conversationId]: [...users, userId] };
    });
  }, []);

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

  return useMemo(
    () => ({
      typingUsers,
      addTypingUser,
      removeTypingUser,
    }),
    [typingUsers, addTypingUser, removeTypingUser],
  );
}
