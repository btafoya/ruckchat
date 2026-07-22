import { useCallback, useMemo, useState } from 'react';

export type PresenceStatus = 'online' | 'offline';

export interface PresenceState {
  presence: Record<string, PresenceStatus>;
  setUserPresence: (userId: string, status: PresenceStatus) => void;
}

export function usePresence(): PresenceState {
  const [presence, setPresence] = useState<Record<string, PresenceStatus>>({});

  const setUserPresence = useCallback((userId: string, status: PresenceStatus) => {
    setPresence((prev) => ({
      ...prev,
      [userId]: status,
    }));
  }, []);

  return useMemo(
    () => ({
      presence,
      setUserPresence,
    }),
    [presence, setUserPresence],
  );
}
