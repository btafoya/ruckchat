import { useMemo } from 'react';
import type { JSX } from 'react';
import { useParams } from 'react-router-dom';
import {
  useChannelContext,
  useDirectMessageContext,
  useMessageContext,
  useOrganizationContext,
  useSessionContext,
  useTypingContext,
} from '../context';
import { Composer } from './Composer';
import { MessageItem } from './MessageItem';
import { ThreadPane } from './ThreadPane';

export function MessagePane(): JSX.Element {
  const params = useParams<{
    organizationId?: string;
    channelId?: string;
    dmId?: string;
    messageId?: string;
  }>();
  const { session } = useSessionContext();
  const { organizations } = useOrganizationContext();
  const { channels } = useChannelContext();
  const { conversations } = useDirectMessageContext();
  const {
    messages,
    isLoading,
    isLoadingMore,
    hasMore,
    loadMore,
  } = useMessageContext();
  const { typingUsers } = useTypingContext();

  const organization = organizations.find((o) => o.id === params.organizationId);
  const channel = channels.find((c) => c.id === params.channelId);
  const conversation = conversations.find((c) => c.id === params.dmId);
  const conversationType = params.channelId ? 'channel' : params.dmId ? 'direct_message' : undefined;
  const conversationId = params.channelId ?? params.dmId;

  const title = useMemo(() => {
    if (channel) {
      return `# ${channel.name}`;
    }
    if (conversation) {
      const others = conversation.member_ids.filter((id) => id !== session?.user.id);
      return `DM: ${others.length > 0 ? others.join(', ') : 'You'}`;
    }
    return null;
  }, [channel, conversation, session?.user.id]);

  const typingList = useMemo(() => {
    if (!conversationId) {
      return [];
    }
    const users = typingUsers[conversationId] ?? [];
    return users.filter((id) => id !== session?.user.id);
  }, [conversationId, typingUsers, session?.user.id]);

  if (!organization) {
    return (
      <div className="flex flex-1 items-center justify-center bg-gray-900 text-gray-400">
        Select an organization from the sidebar.
      </div>
    );
  }

  if (!conversationId || !conversationType || (!channel && !conversation)) {
    return (
      <div className="flex flex-1 items-center justify-center bg-gray-900 text-gray-400">
        Select a channel or direct message in {organization.name}.
      </div>
    );
  }

  return (
    <section className="relative flex flex-1 flex-col overflow-hidden" aria-label="Messages">
      <header className="border-b border-gray-700 px-6 py-4">
        <h1 className="text-lg font-semibold text-white">{title}</h1>
        {channel?.topic && <p className="text-sm text-gray-400">{channel.topic}</p>}
      </header>

      <div className="flex flex-1 flex-col overflow-y-auto p-4">
        {hasMore && (
          <button
            type="button"
            onClick={() => void loadMore()}
            disabled={isLoadingMore}
            className="mb-3 self-center rounded-md bg-gray-700 px-3 py-1 text-xs text-white hover:bg-gray-600 disabled:opacity-50"
          >
            {isLoadingMore ? 'Loading...' : 'Load more history'}
          </button>
        )}

        {isLoading && messages.length === 0 && (
          <div className="text-gray-400">Loading messages...</div>
        )}
        {messages.length === 0 && !isLoading && (
          <div className="text-gray-500">No messages yet.</div>
        )}
        <ul className="flex flex-col gap-3">
          {messages.map((message) => (
            <li key={message.id}>
              <MessageItem message={message} organizationId={organization.id} />
            </li>
          ))}
        </ul>

        {typingList.length > 0 && (
          <div className="mt-2 text-xs italic text-gray-400">
            {typingList.join(', ')} {typingList.length === 1 ? 'is' : 'are'} typing...
          </div>
        )}
      </div>

      <Composer
        conversationType={conversationType}
        conversationId={conversationId}
        organizationId={organization.id}
      />

      {params.messageId && <ThreadPane />}
    </section>
  );
}
