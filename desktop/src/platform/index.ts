import type { ComponentType } from 'react';
import type { RuckChatApi } from '../api';
import type { ServerEvent } from '../api/events';

/**
 * Options passed to the platform notification hook.
 */
export interface NotificationOptions {
  /** Authenticated user id used to filter notifications. */
  userId: string;
  /** Whether notifications are enabled in settings. */
  enabled: boolean;
  /** API client used by web push to subscribe/unsubscribe. */
  api?: RuckChatApi;
  /** Session token used by web push requests. */
  token?: string;
}

/**
 * Result returned to the composer after a platform-specific file upload has been
 * recorded or performed.
 */
export interface SelectedFile {
  /** File identifier that can be attached to a message. */
  id: string;
  /** Human-readable file name. */
  name: string;
}

/**
 * Props accepted by every platform file picker component.
 */
export interface FilePickerProps {
  /** API client used to record or upload the selected files. */
  api: RuckChatApi;
  /** Session token for the authenticated user. */
  token: string;
  /** Organization that owns the uploaded files. */
  organizationId: string;
  /** Called when files have been selected and are ready to attach. */
  onFilesSelected: (files: SelectedFile[]) => void;
  /** Whether the picker is disabled. */
  disabled?: boolean;
}

/**
 * State returned by the platform notification hook.
 */
export interface NotificationState {
  /** Optionally shows a notification for a server event. */
  maybeNotify: (event: ServerEvent) => Promise<void>;
  /** Requests notification permission from the OS or browser. */
  request: () => Promise<void>;
}

/**
 * Platform-specific integrations consumed by the shared UI shell.
 */
export interface Platform {
  /** Reflects unread count in the system tray (desktop only). */
  useTray: (options: { unreadCount: number; enabled: boolean }) => void;
  /** Handles platform deep-link URLs on startup (desktop only). */
  useDeepLink: () => void;
  /** Requests permission and displays notifications. */
  useNotifications: (options: NotificationOptions) => NotificationState;
  /**
   * Component that lets the user select files for attachment. Undefined when
   * the platform does not support file attachments.
   */
  FilePicker: ComponentType<FilePickerProps> | undefined;
}
