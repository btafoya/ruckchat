import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { JSX } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { createApi } from '../api';
import type { Message } from '../api';
import { useDirectMessageContext, useMessageContext, useRealtimeContext, useSessionContext } from '../context';

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
  const { conversations } = useDirectMessageContext();
  const api = useMemo(() => createApi(), []);

  const [content, setContent] = useState(() => {
    try {
      return localStorage.getItem(DRAFT_KEY(conversationId)) ?? '';
    } catch {
      return '';
    }
  });
  const [showPreview, setShowPreview] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [pendingFiles, setPendingFiles] = useState<Array<{ id: string; name: string }>>([]);
  const [mentionQuery, setMentionQuery] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const lastTypingRef = useRef(0);

  useEffect(() => {
    try {
      if (content.trim()) {
        localStorage.setItem(DRAFT_KEY(conversationId), content);
      } else {
        localStorage.removeItem(DRAFT_KEY(conversationId));
      }
    } catch {
      // ignore storage failures
    }
  }, [content, conversationId]);

  const candidateIds = useMemo(() => {
    const ids = new Set<string>();
    for (const conversation of conversations) {
      for (const memberId of conversation.member_ids) {
        if (memberId !== session?.user.id) {
          ids.add(memberId);
        }
      }
    }
    return Array.from(ids);
  }, [conversations, session?.user.id]);

  const sendTyping = useCallback(() => {
    const now = Date.now();
    if (now - lastTypingRef.current < TYPING_DEBOUNCE_MS) {
      return;
    }
    lastTypingRef.current = now;
    const message = {
      type: 'typing',
      conversation_id: conversationId,
      conversation_type: conversationType,
    } as const;
    sendWs(message);
  }, [conversationId, conversationType, sendWs]);

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLTextAreaElement>) => {
      const value = event.target.value;
      setContent(value);
      sendTyping();

      const lastWord = value.split(/\s+/).pop() ?? '';
      if (lastWord.startsWith('@')) {
        setMentionQuery(lastWord.slice(1));
      } else {
        setMentionQuery(null);
      }
    },
    [sendTyping],
  );

  const insertMention = useCallback((userId: string) => {
    const words = content.split(/\s+/);
    words[words.length - 1] = `@${userId}`;
    const next = `${words.join(' ')} `;
    setContent(next);
    setMentionQuery(null);
    textareaRef.current?.focus();
  }, [content]);

  const filteredCandidates = useMemo(() => {
    if (!mentionQuery) {
      return [];
    }
    return candidateIds.filter((id) => id.toLowerCase().includes(mentionQuery.toLowerCase())).slice(0, 5);
  }, [candidateIds, mentionQuery]);

  const handleFileSelect = useCallback(async () => {
    if (!session) {
      return;
    }

    let selected: string | string[] | null = null;
    try {
      selected = await open({
        multiple: true,
      });
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
          const response = await api.files.recordUpload(session.token, {
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

    setPendingFiles((prev) => [...prev, ...(recorded.filter(Boolean) as Array<{ id: string; name: string }>)]);
  }, [api, organizationId, session]);

  const removePendingFile = useCallback((fileId: string) => {
    setPendingFiles((prev) => prev.filter((f) => f.id !== fileId));
  }, []);

  const handleSubmit = useCallback(async () => {
    if (!content.trim() || isSending) {
      return;
    }
    setIsSending(true);
    try {
      const fileIds = pendingFiles.map((f) => f.id);
      const sent = await sendMessage(content, parentId, fileIds);
      if (sent) {
        setContent('');
        setPendingFiles([]);
        setShowPreview(false);
        onSent?.(sent);
      }
    } finally {
      setIsSending(false);
    }
  }, [content, isSending, onSent, parentId, pendingFiles, sendMessage]);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        void handleSubmit();
      }
    },
    [handleSubmit],
  );

  const previewNodes = useMemo(() => {
    if (!showPreview) {
      return null;
    }
    return (
      <div className="min-h-[6rem] whitespace-pre-wrap rounded-md border border-gray-600 bg-gray-900 p-3 text-sm text-gray-200">
        {content || <span className="text-gray-500">Nothing to preview</span>}
      </div>
    );
  }, [showPreview, content]);

  return (
    <div className="flex flex-col gap-2 border-t border-gray-700 bg-gray-800 p-3">
      {pendingFiles.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {pendingFiles.map((file) => (
            <span
              key={file.id}
              className="flex items-center gap-1 rounded-full bg-gray-700 px-2 py-1 text-xs text-gray-200"
            >
              {file.name}
              <button
                type="button"
                onClick={() => removePendingFile(file.id)}
                className="text-gray-400 hover:text-white"
                aria-label={`Remove ${file.name}`}
              >
                ×
              </button>
            </span>
          ))}
        </div>
      )}

      {showPreview ? (
        previewNodes
      ) : (
        <div className="relative">
          <textarea
            ref={textareaRef}
            value={content}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            disabled={isSending}
            className="h-24 w-full resize-none rounded-md border border-gray-600 bg-gray-900 p-3 text-sm text-white placeholder-gray-500 focus:border-green-500 focus:outline-none disabled:opacity-50"
          />
          {mentionQuery !== null && filteredCandidates.length > 0 && (
            <ul className="absolute bottom-full left-0 z-10 mb-1 w-64 rounded-md border border-gray-600 bg-gray-800 py-1 shadow-lg">
              {filteredCandidates.map((id) => (
                <li key={id}>
                  <button
                    type="button"
                    onClick={() => insertMention(id)}
                    className="w-full px-3 py-1 text-left text-sm text-gray-200 hover:bg-gray-700"
                  >
                    @{id}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => void handleFileSelect()}
            disabled={isSending}
            className="rounded-md px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 disabled:opacity-50"
          >
            Attach
          </button>
          <button
            type="button"
            onClick={() => setShowPreview((p) => !p)}
            className="rounded-md px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700"
          >
            {showPreview ? 'Edit' : 'Preview'}
          </button>
        </div>
        <button
          type="button"
          onClick={() => void handleSubmit()}
          disabled={!content.trim() || isSending}
          className="rounded-md bg-green-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-green-500 disabled:opacity-50"
        >
          {isSending ? 'Sending...' : 'Send'}
        </button>
      </div>
    </div>
  );
}
