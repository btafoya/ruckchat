import type { JSX } from 'react';
import type { FilePickerProps, Platform, SelectedFile } from '../platform';

/** Mock file picker for tests. It records selected files from a data-testid attribute. */
export function MockFilePicker({
  onFilesSelected,
  disabled,
}: FilePickerProps): JSX.Element {
  return (
    <button
      type="button"
      data-testid="mock-file-picker"
      data-disabled={disabled}
      onClick={() => onFilesSelected([{ id: 'file-1', name: 'notes.txt' }])}
    >
      Attach
    </button>
  );
}

/** No-op tray hook for tests. */
export function mockUseTray(): void {
  // no-op
}

/** No-op deep-link hook for tests. */
export function mockUseDeepLink(): void {
  // no-op
}

/** No-op notification hook for tests. */
export function mockUseNotifications(): {
  maybeNotify: () => Promise<void>;
  request: () => Promise<void>;
} {
  return {
    maybeNotify: async () => {},
    request: async () => {},
  };
}

/** Mock platform implementation suitable for component tests. */
export const mockPlatform: Platform = {
  useTray: mockUseTray,
  useDeepLink: mockUseDeepLink,
  useNotifications: mockUseNotifications,
  FilePicker: MockFilePicker,
};
