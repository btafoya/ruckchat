import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { ChangeEvent, JSX } from 'react';
import { EditorContent, ReactRenderer, useEditor } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Image from '@tiptap/extension-image';
import Mention from '@tiptap/extension-mention';
import Placeholder from '@tiptap/extension-placeholder';
import suggestion from '@tiptap/suggestion';
import SpellcheckerExtension from '@farscrl/tiptap-extension-spellchecker';
import tippy, { type Instance } from 'tippy.js';
import 'tippy.js/dist/tippy.css';
import { createApi } from '../api';
import type { Message } from '../api';
import {
  useMessageContext,
  usePlatform,
  useRealtimeContext,
  useSessionContext,
  useSettingsContext,
} from '../context';
import { MentionList, type MentionItem, type MentionListHandle, type MentionListProps } from './MentionList';
import { MessageContent } from './MessageContent';
import { SpellingProofreader } from '../spelling/SpellingProofreader';

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
  const { sendMessage, editingMessage, cancelEdit, saveEdit } = useMessageContext();
  const platform = usePlatform();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);

  const [isSending, setIsSending] = useState(false);
  const [pendingFiles, setPendingFiles] = useState<Array<{ id: string; name: string }>>([]);
  const [showPreview, setShowPreview] = useState(false);
  const [isUploadingImage, setIsUploadingImage] = useState(false);
  const lastTypingRef = useRef(0);
  const imageInputRef = useRef<HTMLInputElement>(null);

  const proofreader = useMemo(
    () => new SpellingProofreader(api, () => session?.token),
    [api, session],
  );

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
      SpellcheckerExtension.configure({ proofreader }),
      Image.configure({ HTMLAttributes: { class: 'max-w-full h-auto max-h-80 rounded-md' } }),
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
    // Tiptap v3 defaults to no re-render on transactions (unlike v2); without
    // this, `editor.isEmpty` below is read once and never refreshed as the
    // user types, so the Send button stays stuck in its initial disabled state.
    shouldRerenderOnTransaction: true,
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
    if (editingMessage) {
      let parsed: Record<string, unknown>;
      try {
        const json = JSON.parse(editingMessage.content) as unknown;
        parsed =
          json && typeof json === 'object' && (json as { type?: string }).type === 'doc'
            ? (json as Record<string, unknown>)
            : emptyDoc();
      } catch {
        parsed = emptyDoc();
      }
      editor.commands.setContent(parsed);
      editor.commands.focus('end');
      return;
    }
    editor.commands.setContent(loadDraft(conversationId));
  }, [editor, conversationId, editingMessage]);

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
      if (editingMessage) {
        await saveEdit(content);
        return;
      }
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
  }, [editor, isSending, onSent, parentId, pendingFiles, sendMessage, editingMessage, saveEdit]);

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

  const handleImageFileChange = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file || !session) {
      return;
    }
    setIsUploadingImage(true);
    try {
      const uploaded = await api.files.uploadFile(session.token, organizationId, file);
      editor
        .chain()
        .focus()
        .setImage({ src: `${apiUrl}/files/${uploaded.id}/content`, alt: uploaded.file_name })
        .run();
    } catch {
      // ignore upload failures; the user can retry
    } finally {
      setIsUploadingImage(false);
    }
  };

  const toolbarActions: Array<{ key: string; label: string; title: string; active: boolean; run: () => void }> = [
    { key: 'bold', label: 'B', title: 'Bold', active: editor.isActive('bold'), run: () => editor.chain().focus().toggleBold().run() },
    { key: 'italic', label: 'I', title: 'Italic', active: editor.isActive('italic'), run: () => editor.chain().focus().toggleItalic().run() },
    { key: 'strike', label: 'S', title: 'Strikethrough', active: editor.isActive('strike'), run: () => editor.chain().focus().toggleStrike().run() },
    { key: 'code', label: '<>', title: 'Inline code', active: editor.isActive('code'), run: () => editor.chain().focus().toggleCode().run() },
    { key: 'bulletList', label: '•', title: 'Bullet list', active: editor.isActive('bulletList'), run: () => editor.chain().focus().toggleBulletList().run() },
    { key: 'orderedList', label: '1.', title: 'Numbered list', active: editor.isActive('orderedList'), run: () => editor.chain().focus().toggleOrderedList().run() },
    { key: 'blockquote', label: '❝', title: 'Blockquote', active: editor.isActive('blockquote'), run: () => editor.chain().focus().toggleBlockquote().run() },
    { key: 'codeBlock', label: '{}', title: 'Code block', active: editor.isActive('codeBlock'), run: () => editor.chain().focus().toggleCodeBlock().run() },
  ];

  return (
    <div className="flex flex-col gap-2 border-t border-border bg-surface p-3">
      {!showPreview && (
        <div className="flex flex-wrap items-center gap-1">
          {toolbarActions.map((action) => (
            <button
              key={action.key}
              type="button"
              onMouseDown={(e) => e.preventDefault()}
              onClick={action.run}
              disabled={isSending}
              title={action.title}
              aria-pressed={action.active}
              className={`rounded px-2 py-1 text-xs font-semibold hover:bg-surface-elevated disabled:opacity-50 ${
                action.active ? 'bg-surface-elevated text-accent' : 'text-text'
              }`}
            >
              {action.label}
            </button>
          ))}
          <button
            type="button"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => imageInputRef.current?.click()}
            disabled={isSending || isUploadingImage}
            title="Insert image"
            className="rounded px-2 py-1 text-xs font-semibold text-text hover:bg-surface-elevated disabled:opacity-50"
          >
            {isUploadingImage ? 'Uploading...' : 'Image'}
          </button>
          <input
            ref={imageInputRef}
            type="file"
            accept="image/*"
            className="hidden"
            onChange={(e) => void handleImageFileChange(e)}
          />
        </div>
      )}

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
        <div className="flex items-center gap-2">
          {editingMessage && (
            <button
              type="button"
              onClick={cancelEdit}
              disabled={isSending}
              className="rounded-md px-3 py-1.5 text-sm text-text hover:bg-surface-elevated disabled:opacity-50"
            >
              Cancel
            </button>
          )}
          <button
            type="button"
            onClick={() => void handleSubmit()}
            disabled={editor.isEmpty || isSending}
            className="rounded-md bg-accent px-4 py-1.5 text-sm font-semibold text-text-inverse hover:bg-accent-hover disabled:opacity-50"
          >
            {editingMessage ? 'Save' : isSending ? 'Sending...' : 'Send'}
          </button>
        </div>
      </div>
    </div>
  );
}

