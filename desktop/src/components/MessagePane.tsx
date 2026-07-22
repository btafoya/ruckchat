import type { JSX } from 'react';
import { useParams } from 'react-router-dom';
import { useChannelContext, useMessageContext, useOrganizationContext } from '../context';

export function MessagePane(): JSX.Element {
  const params = useParams();
  const { organizations } = useOrganizationContext();
  const { channels } = useChannelContext();
  const { messages, isLoading } = useMessageContext();

  const organization = organizations.find((o) => o.id === params.organizationId);
  const channel = channels.find((c) => c.id === params.channelId);

  if (!organization) {
    return (
      <div className="flex flex-1 items-center justify-center bg-gray-900 text-gray-400">
        Select an organization from the sidebar.
      </div>
    );
  }

  if (!channel) {
    return (
      <div className="flex flex-1 items-center justify-center bg-gray-900 text-gray-400">
        Select a channel in {organization.name}.
      </div>
    );
  }

  return (
    <section className="flex flex-1 flex-col overflow-hidden" aria-label="Messages">
      <header className="border-b border-gray-700 px-6 py-4">
        <h1 className="text-lg font-semibold text-white"># {channel.name}</h1>
        {channel.topic && <p className="text-sm text-gray-400">{channel.topic}</p>}
      </header>
      <div className="flex flex-1 flex-col overflow-y-auto p-4">
        {isLoading && <div className="text-gray-400">Loading messages...</div>}
        {messages.length === 0 && !isLoading && (
          <div className="text-gray-500">No messages yet.</div>
        )}
        <ul className="flex flex-col gap-3">
          {messages.map((message) => (
            <li key={message.id} className="text-sm text-gray-200">
              <span className="font-semibold text-green-400">{message.author_id}</span>: {message.content}
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
