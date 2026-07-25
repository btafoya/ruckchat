import { useMemo, type JSX } from 'react';
import { Link, useParams } from 'react-router-dom';
import {
  useChannelContext,
  useDirectMessageContext,
  useOrgMemberContext,
  useOrganizationContext,
  useReadStateContext,
  useSessionContext,
} from '../context';

interface ConversationRow {
  id: string;
  type: 'channel' | 'dm';
  label: string;
  href: string;
  unread: number;
}

export function OrgHome(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  const { session } = useSessionContext();
  const { organizations } = useOrganizationContext();
  const { channels, isLoading: channelsLoading } = useChannelContext();
  const { conversations, isLoading: dmsLoading } = useDirectMessageContext();
  const { members: orgMembers } = useOrgMemberContext();
  const { counts } = useReadStateContext();

  const organization = organizations.find((o) => o.id === organizationId);

  const rows = useMemo<ConversationRow[]>(() => {
    if (!organizationId) {
      return [];
    }
    const result: ConversationRow[] = [];
    for (const channel of channels.filter((c) => !c.archived_at)) {
      result.push({
        id: channel.id,
        type: 'channel',
        label: `# ${channel.name}`,
        href: `/org/${organizationId}/channel/${channel.id}`,
        unread: counts[channel.id] ?? 0,
      });
    }
    for (const conversation of conversations) {
      const others = conversation.member_ids.filter((id) => id !== session?.user.id);
      const names = others.map(
        (id) => orgMembers.find((m) => m.id === id)?.display_name ?? id,
      );
      const label = names.length > 0 ? names.join(', ') : 'You';
      result.push({
        id: conversation.id,
        type: 'dm',
        label,
        href: `/org/${organizationId}/dm/${conversation.id}`,
        unread: counts[conversation.id] ?? 0,
      });
    }
    return result.sort((a, b) => b.unread - a.unread);
  }, [channels, conversations, counts, organizationId, session?.user.id, orgMembers]);

  const totalUnread = rows.reduce((sum, row) => sum + row.unread, 0);

  if (!organization) {
    return (
      <div className="flex h-full items-center justify-center bg-bg p-6 text-text">
        <p>Organization not found.</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto bg-bg p-6 text-text">
      <header className="mb-6">
        <h1 className="text-2xl font-semibold">{organization.name}</h1>
        <p className="text-sm text-text-muted">
          {totalUnread > 0 ? `${totalUnread} unread conversation${totalUnread === 1 ? '' : 's'}` : 'All caught up'}
        </p>
      </header>

      {channelsLoading || dmsLoading ? (
        <div className="text-sm text-text-muted">Loading conversations...</div>
      ) : rows.length === 0 ? (
        <div className="rounded-md border border-border bg-surface p-6 text-sm text-text-muted">
          No conversations yet. Create a channel or start a direct message from the sidebar.
        </div>
      ) : (
        <nav aria-label="Unread conversations" className="space-y-1">
          {rows.map((row) => (
            <Link
              key={row.id}
              to={row.href}
              className="flex items-center justify-between rounded-md border border-border bg-surface px-4 py-3 hover:bg-surface-elevated"
            >
              <span className="text-sm font-medium">{row.label}</span>
              {row.unread > 0 && (
                <span className="rounded-full bg-accent px-2 py-0.5 text-xs font-semibold text-text-inverse">
                  {row.unread > 99 ? '99+' : row.unread}
                </span>
              )}
            </Link>
          ))}
        </nav>
      )}
    </div>
  );
}
