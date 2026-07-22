import { useCallback, useEffect, useRef } from 'react';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import type { ServerEvent } from '../api/events';

export interface NotificationOptions {
  userId: string;
  enabled: boolean;
}

export interface NotificationState {
  maybeNotify: (event: ServerEvent) => Promise<void>;
  request: () => Promise<void>;
}

function shouldNotify(event: ServerEvent, userId: string): boolean {
  if (event.type !== 'message.created') {
    return false;
  }
  const { message } = event;
  if (message.author_id === userId) {
    return false;
  }
  if (message.conversation_type === 'direct_message') {
    return true;
  }
  const mention = `@${userId}`;
  return message.content.includes(mention);
}

export function useNotifications(options: NotificationOptions): NotificationState {
  const permissionRef = useRef(false);

  const request = useCallback(async () => {
    if (!options.enabled) {
      return;
    }
    try {
      let granted = await isPermissionGranted();
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === 'granted';
      }
      permissionRef.current = granted;
    } catch {
      permissionRef.current = false;
    }
  }, [options.enabled]);

  useEffect(() => {
    void request();
  }, [request]);

  const maybeNotify = useCallback(
    async (event: ServerEvent) => {
      if (!options.enabled || !shouldNotify(event, options.userId)) {
        return;
      }
      try {
        if (!permissionRef.current) {
          await request();
        }
        if (!permissionRef.current) {
          return;
        }
        if (event.type === 'message.created') {
          sendNotification({
            title: 'RuckChat',
            body: event.message.content,
          });
        }
      } catch {
        // ignore notification failures
      }
    },
    [options.enabled, options.userId, request],
  );

  return {
    maybeNotify,
    request,
  };
}
