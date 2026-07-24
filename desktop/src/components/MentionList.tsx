import { forwardRef, useEffect, useImperativeHandle, useState } from 'react';
import type { JSX } from 'react';

export interface MentionItem {
  id: string;
  label: string;
}

export interface MentionListProps {
  items: MentionItem[];
  command: (item: MentionItem) => void;
}

export interface MentionListHandle {
  onKeyDown: (event: KeyboardEvent) => boolean;
}

export const MentionList = forwardRef<MentionListHandle, MentionListProps>(
  ({ items, command }, ref): JSX.Element => {
    const [selectedIndex, setSelectedIndex] = useState(0);

    useEffect(() => {
      setSelectedIndex(0);
    }, [items]);

    useImperativeHandle(ref, () => ({
      onKeyDown: (event: KeyboardEvent): boolean => {
        if (event.key === 'ArrowUp') {
          event.preventDefault();
          setSelectedIndex((prev) => (prev + items.length - 1) % items.length);
          return true;
        }
        if (event.key === 'ArrowDown') {
          event.preventDefault();
          setSelectedIndex((prev) => (prev + 1) % items.length);
          return true;
        }
        if (event.key === 'Enter' || event.key === 'Tab') {
          event.preventDefault();
          const item = items[selectedIndex];
          if (item) {
            command(item);
          }
          return true;
        }
        return false;
      },
    }));

    if (items.length === 0) {
      return (
        <div className="rounded-md border border-border bg-surface px-3 py-2 text-sm text-text-muted shadow-lg">
          No members found
        </div>
      );
    }

    return (
      <ul className="max-h-60 w-64 overflow-y-auto rounded-md border border-border bg-surface py-1 shadow-lg">
        {items.map((item, index) => (
          <li key={item.id}>
            <button
              type="button"
              onClick={() => command(item)}
              className={`w-full px-3 py-1.5 text-left text-sm ${
                index === selectedIndex
                  ? 'bg-surface-elevated text-text'
                  : 'text-text hover:bg-surface-elevated'
              }`}
            >
              @{item.label}
            </button>
          </li>
        ))}
      </ul>
    );
  },
);

MentionList.displayName = 'MentionList';
