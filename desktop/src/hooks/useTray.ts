import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface UseTrayOptions {
  unreadCount: number;
  enabled: boolean;
}

export function useTray(options: UseTrayOptions): void {
  const lastCountRef = useRef(-1);

  useEffect(() => {
    if (!options.enabled) {
      return;
    }
    if (lastCountRef.current === options.unreadCount) {
      return;
    }
    lastCountRef.current = options.unreadCount;
    void invoke('set_unread_count', { count: options.unreadCount }).catch(() => {
      // ignore tray update failures
    });
  }, [options.enabled, options.unreadCount]);
}
