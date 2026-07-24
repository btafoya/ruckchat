import { useEffect, useMemo } from 'react';
import type { JSX } from 'react';
import { NavLink, useParams } from 'react-router-dom';
import { useMessageContext } from '../context';
import { Composer } from './Composer';
import { MessageItem } from './MessageItem';

export function ThreadPane(): JSX.Element {
  const params = useParams<{
    organizationId?: string;
    channelId?: string;
    dmId?: string;
    messageId?: string;
  }>();
  const organizationId = params.organizationId;
  const channelId = params.channelId;
  const dmId = params.dmId;
  const messageId = params.messageId;
  const conversationType = channelId ? 'channel' : 'direct_message';
  const conversationId = channelId ?? dmId;
  const backPath = channelId
    ? `/org/${organizationId}/channel/${channelId}`
    : `/org/${organizationId}/dm/${dmId}`;

  const { messages, threadReplies, threadRepliesLoading, loadThreadReplies } = useMessageContext();

  useEffect(() => {
    if (messageId) {
      void loadThreadReplies(messageId);
    }
  }, [loadThreadReplies, messageId]);

  const parent = useMemo(
    () => messages.find((m) => m.id === messageId),
    [messages, messageId],
  );

  if (!organizationId || !conversationId || !messageId) {
    return <div />;
  }

  return (
    <div className="absolute inset-0 z-20 flex justify-end bg-overlay">
      <section className="flex w-full max-w-md flex-col border-l border-border bg-surface shadow-xl">
        <header className="flex items-center justify-between border-b border-border px-4 py-3">
          <div className="text-sm font-semibold text-text">Thread</div>
          <NavLink
            to={backPath}
            className="text-sm text-text hover:text-text-muted"
          >
            Close
          </NavLink>
        </header>

        <div className="flex flex-1 flex-col overflow-y-auto p-4">
          {parent && <MessageItem message={parent} organizationId={organizationId} showReplyButton={false} />}
          {!parent && !threadRepliesLoading && (
            <div className="text-sm text-text-muted">Parent message not found.</div>
          )}

          <div className="my-2 border-t border-border" />

          {threadRepliesLoading && <div className="text-sm text-text-muted">Loading replies...</div>}
          {threadReplies.length === 0 && !threadRepliesLoading && (
            <div className="text-sm text-text-muted">No replies yet.</div>
          )}
          <ul className="flex flex-col gap-3">
            {threadReplies.map((reply) => (
              <li key={reply.id}>
                <MessageItem message={reply} organizationId={organizationId} showReplyButton={false} />
              </li>
            ))}
          </ul>
        </div>

        <Composer
          conversationType={conversationType}
          conversationId={conversationId}
          organizationId={organizationId}
          parentId={messageId}
          placeholder="Reply in thread..."
        />
      </section>
    </div>
  );
}
