import { useCallback, useEffect, useRef, type ChangeEvent, type JSX } from 'react';
import type { ServerEvent } from '../api/events';
import type {
  FilePickerProps,
  NotificationOptions as WebNotificationOptions,
  NotificationState,
  Platform,
  SelectedFile,
} from './index';

function supportsNotifications(): boolean {
  return typeof window !== 'undefined' && 'Notification' in window;
}

function supportsServiceWorker(): boolean {
  return typeof navigator !== 'undefined' && 'serviceWorker' in navigator;
}

function bufferToBase64(buffer: ArrayBuffer | ArrayBufferView): string {
  const bytes =
    buffer instanceof ArrayBuffer
      ? new Uint8Array(buffer)
      : new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  return window.btoa(String.fromCharCode(...bytes));
}

function applicationServerKeyMatches(
  subscription: PushSubscription,
  expectedKey: string,
): boolean {
  const rawKey = subscription.options?.applicationServerKey;
  if (!rawKey || typeof rawKey === 'string') {
    return rawKey === expectedKey;
  }
  return bufferToBase64(rawKey as ArrayBuffer | ArrayBufferView) === expectedKey;
}

async function subscribeToPush(
  api: WebNotificationOptions['api'],
  token: WebNotificationOptions['token'],
): Promise<void> {
  if (!supportsServiceWorker() || !api || !token) {
    return;
  }
  try {
    const { public_key: publicKey } = await api.webPush.getVapidKey();
    if (!publicKey) {
      return;
    }
    const registration = await navigator.serviceWorker.ready;
    const existing = await registration.pushManager.getSubscription();
    if (existing) {
      if (applicationServerKeyMatches(existing, publicKey)) {
        const json = existing.toJSON();
        if (json.endpoint && json.keys?.p256dh && json.keys?.auth) {
          await api.webPush.subscribe(token, {
            endpoint: json.endpoint,
            p256dh: json.keys.p256dh,
            auth: json.keys.auth,
          });
        }
        return;
      }
      await existing.unsubscribe();
    }
    const subscription = await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: publicKey,
    });
    const json = subscription.toJSON();
    if (json.endpoint && json.keys?.p256dh && json.keys?.auth) {
      await api.webPush.subscribe(token, {
        endpoint: json.endpoint,
        p256dh: json.keys.p256dh,
        auth: json.keys.auth,
      });
    }
  } catch (err) {
    console.warn('Failed to subscribe to web push', err);
  }
}

async function unsubscribeFromPush(
  api: WebNotificationOptions['api'],
  token: WebNotificationOptions['token'],
): Promise<void> {
  if (!supportsServiceWorker() || !api || !token) {
    return;
  }
  try {
    const registration = await navigator.serviceWorker.ready;
    const existing = await registration.pushManager.getSubscription();
    if (existing) {
      const json = existing.toJSON();
      await existing.unsubscribe();
      if (json.endpoint) {
        await api.webPush.unsubscribe(token, { endpoint: json.endpoint });
      }
    }
  } catch (err) {
    console.warn('Failed to unsubscribe from web push', err);
  }
}

async function showNotification(title: string, options?: NotificationOptions): Promise<void> {
  if (!supportsServiceWorker()) {
    return;
  }
  try {
    const registration = await navigator.serviceWorker.ready;
    await registration.showNotification(title, options);
  } catch {
    // ignore notification failures
  }
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
  return message.content.includes(`@${userId}`);
}

/** No-op tray hook for browsers. */
export function useTray(): void {
  // Browsers have no system tray API.
}

/** No-op deep-link hook for browsers; URL routing is handled by React Router. */
export function useDeepLink(): void {
  // Deep links are not supported in the browser.
}

/** Web Push-backed notification hook. */
export function useNotifications(options: WebNotificationOptions): NotificationState {
  const permissionRef = useRef(false);

  const request = useCallback(async () => {
    if (!options.enabled || !supportsNotifications()) {
      return;
    }
    try {
      let granted = Notification.permission === 'granted';
      if (!granted && Notification.permission !== 'denied') {
        const permission = await Notification.requestPermission();
        granted = permission === 'granted';
      }
      permissionRef.current = granted;
      if (granted) {
        await subscribeToPush(options.api, options.token);
      }
    } catch {
      permissionRef.current = false;
    }
  }, [options.enabled, options.api, options.token]);

  useEffect(() => {
    void request();
  }, [request]);

  useEffect(() => {
    if (!options.enabled) {
      void unsubscribeFromPush(options.api, options.token);
    }
  }, [options.enabled, options.api, options.token]);

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
          await showNotification('RuckChat', {
            body: event.message.content,
            icon: '/icons/icon-192x192.png',
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

function WebFilePicker({
  api,
  token,
  organizationId,
  onFilesSelected,
  disabled,
}: FilePickerProps): JSX.Element {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleChange = useCallback(
    async (event: ChangeEvent<HTMLInputElement>) => {
      const files = event.target.files;
      if (!files || files.length === 0) {
        return;
      }

      const uploaded: SelectedFile[] = [];
      for (const file of Array.from(files)) {
        try {
          const response = await api.files.uploadFile(token, organizationId, file);
          uploaded.push({ id: response.id, name: file.name });
        } catch (err) {
          console.warn('Failed to upload file', err);
        }
      }

      onFilesSelected(uploaded);
      if (inputRef.current) {
        inputRef.current.value = '';
      }
    },
    [api, token, organizationId, onFilesSelected],
  );

  const handleClick = useCallback(() => {
    inputRef.current?.click();
  }, []);

  return (
    <>
      <input
        ref={inputRef}
        type="file"
        multiple
        className="hidden"
        onChange={handleChange}
        disabled={disabled}
      />
      <button
        type="button"
        onClick={() => void handleClick()}
        disabled={disabled}
        className="rounded-md px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 disabled:opacity-50"
      >
        Attach
      </button>
    </>
  );
}

/** Web platform implementation using browser APIs. */
export const webPlatform: Platform = {
  useTray,
  useDeepLink,
  useNotifications,
  FilePicker: WebFilePicker,
};
