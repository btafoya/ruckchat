import type { JSX } from 'react';

interface ConfirmationModalProps {
  title: string;
  message: string;
  confirmLabel: string;
  confirmDanger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmationModal({
  title,
  message,
  confirmLabel,
  confirmDanger = false,
  onConfirm,
  onCancel,
}: ConfirmationModalProps): JSX.Element {
  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-overlay p-4"
      onClick={onCancel}
      role="presentation"
    >
      <div
        className="w-full max-w-sm rounded-lg bg-surface p-6 shadow-lg"
        onClick={(e) => e.stopPropagation()}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
      >
        <h3 id="confirm-title" className="text-lg font-semibold text-text">
          {title}
        </h3>
        <p className="mt-2 text-sm text-text-muted">{message}</p>
        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-md px-4 py-2 text-sm text-text hover:bg-surface-elevated"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={`rounded-md px-4 py-2 text-sm font-semibold text-text-inverse ${
              confirmDanger
                ? 'bg-danger hover:bg-danger-hover'
                : 'bg-accent hover:bg-accent-hover'
            }`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
