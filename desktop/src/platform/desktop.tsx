import { useCallback, useMemo, useRef, type JSX } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useDeepLink, useNotifications, useTray } from '../hooks';
import type { RuckChatApi } from '../api';
import type { FilePickerProps, Platform, SelectedFile } from './index';

function DesktopFilePicker({
  api,
  token,
  organizationId,
  onFilesSelected,
  disabled,
}: FilePickerProps): JSX.Element {
  const selectingRef = useRef(false);

  const handleClick = useCallback(async () => {
    if (selectingRef.current || disabled) {
      return;
    }
    selectingRef.current = true;
    try {
      let selected: string | string[] | null = null;
      try {
        selected = await open({ multiple: true });
      } catch (err) {
        console.warn('Failed to open file dialog', err);
        return;
      }
      if (!selected) {
        return;
      }
      const paths = Array.isArray(selected) ? selected : [selected];

      const recorded = await Promise.all(
        paths.map(async (path) => {
          const fileName = path.split('/').pop() ?? path.split('\\').pop() ?? path;
          try {
            const response = await api.files.recordUpload(token, {
              organization_id: organizationId,
              file_name: fileName,
              mime_type: 'application/octet-stream',
              size_bytes: 0,
              storage_path: path,
            });
            return { id: response.id, name: fileName };
          } catch (err) {
            console.warn('Failed to record file upload', err);
            return null;
          }
        }),
      );

      onFilesSelected(recorded.filter((f): f is SelectedFile => f !== null));
    } finally {
      selectingRef.current = false;
    }
  }, [api, disabled, onFilesSelected, organizationId, token]);

  return (
    <button
      type="button"
      onClick={() => void handleClick()}
      disabled={disabled}
      className="rounded-md px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 disabled:opacity-50"
    >
      Attach
    </button>
  );
}

/** Desktop platform implementation backed by Tauri APIs. */
export const desktopPlatform: Platform = {
  useTray,
  useDeepLink,
  useNotifications,
  FilePicker: DesktopFilePicker,
};
