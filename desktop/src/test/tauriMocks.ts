import { vi } from 'vitest';

export function mockTauriPlugins(): void {
  vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
  }));

  vi.mock('@tauri-apps/plugin-notification', () => ({
    isPermissionGranted: vi.fn().mockResolvedValue(false),
    requestPermission: vi.fn().mockResolvedValue('default'),
    sendNotification: vi.fn(),
  }));

  vi.mock('@tauri-apps/plugin-deep-link', () => ({
    getCurrent: vi.fn().mockResolvedValue(null),
  }));

  vi.mock('@tauri-apps/plugin-dialog', () => ({
    open: vi.fn().mockResolvedValue(null),
    save: vi.fn().mockResolvedValue(null),
  }));
}
