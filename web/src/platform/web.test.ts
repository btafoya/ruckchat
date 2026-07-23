import { createElement, type ComponentType } from 'react';
import { renderToString } from 'react-dom/server';
import { describe, expect, it, vi } from 'vitest';
import { webPlatform, useTray, useDeepLink } from '../../../desktop/src/platform/web';
import type { FilePickerProps } from '../../../desktop/src/platform';

describe('web platform', () => {
  it('exports the expected hooks and a file picker component', () => {
    expect(webPlatform.useTray).toBe(useTray);
    expect(webPlatform.useDeepLink).toBe(useDeepLink);
    expect(webPlatform.FilePicker).toBeDefined();
  });

  it('returns no-op tray and deep-link hooks', () => {
    expect(useTray()).toBeUndefined();
    expect(useDeepLink()).toBeUndefined();
  });

  it('renders the web file picker without crashing', () => {
    const FilePicker = webPlatform.FilePicker as ComponentType<FilePickerProps>;
    const api = {
      files: {
        uploadFile: vi.fn().mockResolvedValue({ id: 'file-1' }),
      },
    } as unknown as FilePickerProps['api'];
    const element = createElement(FilePicker, {
      api,
      token: 'token',
      organizationId: 'org-1',
      onFilesSelected: vi.fn(),
      disabled: false,
    });
    const html = renderToString(element);
    expect(html).toContain('Attach');
  });
});
