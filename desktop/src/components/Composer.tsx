import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { JSX } from 'react';
import { EditorContent, ReactRenderer, useEditor } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Mention from '@tiptap/extension-mention';
import Placeholder from '@tiptap/extension-placeholder';
import suggestion from '@tiptap/suggestion';
import tippy, { type Instance } from 'tippy.js';
import 'tippy.js/dist/tippy.css';
import { createApi } from '../api';
import type { Message } from '../api';
import {
  useMessageContext,
  usePlatform,
  useRealtimeContext,
  useSessionContext,
} from '../context';
import { MentionList, type MentionItem, type MentionListHandle, type MentionListProps } from './MentionList';
import { MessageContent } from './MessageContent';

const TYPING_DEBOUNCE_MS = 1500;
const DRAFT_KEY = (conversationId: string) => `ruckchat_draft_${conversationId}`;

interface ComposerProps {
  conversationType: 'channel' | 'direct_message';
  conversationId: string;
  organizationId: string;
  parentId?: string;
  placeholder?: string;
  onSent?: (message: Message) => void;
}

function emptyDoc() {
  return { type: 'doc', content: [{ type: 'paragraph' }] };
}

function loadDraft(conversationId: string): Record<string, unknown> {
  try {
    const raw = localStorage.getItem(DRAFT_KEY(conversationId));
    if (!raw) {
      return emptyDoc();
    }
    const parsed = JSON.parse(raw) as unknown;
    if (parsed && typeof parsed === 'object' && (parsed as { type?: string }).type === 'doc') {
      return parsed as Record<string, unknown>;
    }
    return emptyDoc();
  } catch {
    return emptyDoc();
  }
}

export function Composer({
  conversationType,
  conversationId,
  organizationId,
  parentId,
  placeholder = 'Type a message...',
  onSent,
}: ComposerProps): JSX.Element {
  const { session } = useSessionContext();
  const { send: sendWs } = useRealtimeContext();
  const { sendMessage } = useMessageContext();
  const platform = usePlatform();
  const api = useMemo(() => createApi(), []);

  const [isSending, setIsSending] = useState(false);
  const [pendingFiles, setPendingFiles] = useState<Array<{ id: string; name: string }>>([]);
  const [showPreview, setShowPreview] = useState(false);
  const lastTypingRef = useRef(0);

  const sendTyping = useCallback(() => {
    const now = Date.now();
    if (now - lastTypingRef.current < TYPING_DEBOUNCE_MS) {
      return;
    }
    lastTypingRef.current = now;
    sendWs({
      type: 'typing',
      conversation_id: conversationId,
      conversation_type: conversationType,
    });
  }, [conversationId, conversationType, sendWs]);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({ hardBreak: { keepMarks: true } }),
      Placeholder.configure({ placeholder }),
      Mention.configure({
        suggestion: {
          char: '@',
          allowSpaces: true,
          startOfLine: false,
          items: async ({ query }) => {
            if (!session || query.trim().length === 0) {
              return [];
            }
            try {
              const users = await api.organizations.searchMembers(
                session.token,
                organizationId,
                query,
              );
              return users
                .filter((u) => u.id !== session.user.id)
                .slice(0, 5)
                .map(
                  (u): MentionItem => ({
                    id: u.id,
                    label: u.display_name || u.email,
                  }),
                );
            } catch {
              return [];
            }
          },
          command: ({ editor, range, props }) => {
            editor
              .chain()
              .focus()
              .deleteRange(range)
              .insertContent({
                type: 'mention',
                attrs: { id: props.id, label: props.label },
              })
              .insertContent(' ')
              .run();
          },
          render: () => {
            let reactRenderer: ReactRenderer<MentionListHandle, MentionListProps>;
            let popup: Instance;
            return {
              onStart: (props) => {
                reactRenderer = new ReactRenderer(MentionList, {
                  props,
                  editor: props.editor,
                });
                popup = tippy(props.editor.view.dom as Element, {
                  getReferenceClientRect: () =>
                    props.clientRect?.() ?? props.editor.view.dom.getBoundingClientRect(),
                  appendTo: () => document.body,
                  content: reactRenderer.element,
                  showOnCreate: true,
                  interactive: true,
                  trigger: 'manual',
                  placement: 'bottom-start',
                });
              },
              onUpdate: (props) => {
                reactRenderer.updateProps(props);
                popup.setProps({
                  getReferenceClientRect: () =>
                    props.clientRect?.() ?? props.editor.view.dom.getBoundingClientRect(),
                });
              },
              onKeyDown: (props) => {
                return reactRenderer.ref?.onKeyDown(props.event) ?? false;
              },
              onExit: () => {
                popup.destroy();
                reactRenderer.destroy();
              },
            };
          },
        },
      }),
    ],
    content: loadDraft(conversationId),
    editorProps: {
      attributes: {
        class:
          'h-24 w-full resize-none rounded-md border border-border bg-bg p-3 text-sm text-text placeholder:text-text-muted focus:border-accent focus:outline-none disabled:opacity-50 overflow-y-auto',
        'aria-label': placeholder,
        role: 'textbox',
        spellcheck: 'true',
      },
    },
    autofocus: false,
  });

  useEffect(() => {
    if (!editor) {
      return;
    }
    const saveDraft = () => {
      const content = editor.isEmpty ? '' : JSON.stringify(editor.getJSON());
      try {
        if (content) {
          localStorage.setItem(DRAFT_KEY(conversationId), content);
        } else {
          localStorage.removeItem(DRAFT_KEY(conversationId));
        }
      } catch {
        // ignore storage failures
      }
    };
    editor.on('update', saveDraft);
    return () => {
      editor.off('update', saveDraft);
    };
  }, [editor, conversationId]);

  useEffect(() => {
    if (!editor) {
      return;
    }
    editor.commands.setContent(loadDraft(conversationId));
  }, [editor, conversationId]);

  useEffect(() => {
    if (!editor) {
      return;
    }
    editor.setEditable(!isSending);
  }, [editor, isSending]);

  useEffect(() => {
    if (!editor) {
      return;
    }
    const onUpdate = () => {
      sendTyping();
    };
    editor.on('update', onUpdate);
    return () => {
      editor.off('update', onUpdate);
    };
  }, [editor, sendTyping]);

  const handleSubmit = useCallback(async () => {
    if (!editor || editor.isEmpty || isSending) {
      return;
    }
    setIsSending(true);
    try {
      const content = JSON.stringify(editor.getJSON());
      const fileIds = pendingFiles.map((f) => f.id);
      const sent = await sendMessage(content, parentId, fileIds);
      if (sent) {
        editor.commands.clearContent();
        setPendingFiles([]);
        setShowPreview(false);
        onSent?.(sent);
      }
    } finally {
      setIsSending(false);
    }
  }, [editor, isSending, onSent, parentId, pendingFiles, sendMessage]);

  useEffect(() => {
    if (!editor || !showPreview) {
      return;
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setShowPreview(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [editor, showPreview]);

  useEffect(() => {
    if (!editor) {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        event.key === 'Enter' &&
        !event.shiftKey &&
        !isSending &&
        event.target === editor.view.dom
      ) {
        event.preventDefault();
        void handleSubmit();
      }
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => {
      window.removeEventListener('keydown', onKeyDown, true);
    };
  }, [editor, handleSubmit, isSending]);

  const previewContent = useMemo(() => {
    if (!editor || editor.isEmpty) {
      return '';
    }
    return JSON.stringify(editor.getJSON());
  }, [editor, showPreview]);

  const removePendingFile = useCallback((fileId: string) => {
    setPendingFiles((prev) => prev.filter((f) => f.id !== fileId));
  }, []);

  if (!editor) {
    return <div className="h-24 w-full rounded-md border border-border bg-bg p-3" />;
  }

  return (
    <div className="flex flex-col gap-2 border-t border-border bg-surface p-3">
      {pendingFiles.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {pendingFiles.map((file) => (
            <span
              key={file.id}
              className="flex items-center gap-1 rounded-full bg-surface-elevated px-2 py-1 text-xs text-text"
            >
              {file.name}
              <button
                type="button"
                onClick={() => removePendingFile(file.id)}
                className="text-text-muted hover:text-text"
                aria-label={`Remove ${file.name}`}
              >
                ×
              </button>
            </span>
          ))}
        </div>
      )}

      {showPreview ? (
        <div className="min-h-[6rem] rounded-md border border-border bg-bg p-3 text-sm text-text">
          {previewContent ? (
            <MessageContent content={previewContent} />
          ) : (
            <span className="text-text-muted">Nothing to preview</span>
          )}
        </div>
      ) : (
        <EditorContent editor={editor} disabled={isSending} />
      )}

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {session && platform.FilePicker && (
            <platform.FilePicker
              api={api}
              token={session.token}
              organizationId={organizationId}
              onFilesSelected={(files) => setPendingFiles((prev) => [...prev, ...files])}
              disabled={isSending}
            />
          )}
          <button
            type="button"
            onClick={() => setShowPreview((p) => !p)}
            className="rounded-md px-3 py-1.5 text-sm text-text hover:bg-surface-elevated"
          >
            {showPreview ? 'Edit' : 'Preview'}
          </button>
        </div>
        <button
          type="button"
          onClick={() => void handleSubmit()}
          disabled={editor.isEmpty || isSending}
          className="rounded-md bg-accent px-4 py-1.5 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
        >
          {isSending ? 'Sending...' : 'Send'}
        </button>
      </div>
    </div>
  );
}

