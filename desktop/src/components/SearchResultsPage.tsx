import { Children, useEffect, useMemo, useState } from 'react';
import type { JSX, ReactNode } from 'react';
import { Link, useParams, useSearchParams } from 'react-router-dom';
import { createApi } from '../api';
import type { SearchResponse } from '../api';
import { useSessionContext, useSettingsContext } from '../context';
import { MessageContent } from './MessageContent';

function ResultSection({
  title,
  empty,
  children,
}: {
  title: string;
  empty: string;
  children: ReactNode;
}): JSX.Element {
  const isEmpty = Children.count(children) === 0;
  return (
    <section>
      <h2 className="mb-2 text-xs font-semibold uppercase tracking-wide text-text-muted">{title}</h2>
      {isEmpty ? (
        <p className="text-sm text-text-muted">{empty}</p>
      ) : (
        <ul className="flex flex-col gap-1">{children}</ul>
      )}
    </section>
  );
}

export function SearchResultsPage(): JSX.Element {
  const { organizationId } = useParams<{ organizationId: string }>();
  const [searchParams] = useSearchParams();
  const query = searchParams.get('q') ?? '';
  const { session } = useSessionContext();
  const { apiUrl } = useSettingsContext();
  const api = useMemo(() => createApi(apiUrl), [apiUrl]);
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!session || !organizationId || !query.trim()) {
      setResults(null);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setError(null);
    api.search
      .search(session.token, organizationId, query)
      .then((response) => {
        if (!cancelled) {
          setResults(response);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Search failed');
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api, session, organizationId, query]);

  if (!organizationId) {
    return <div className="p-4 text-text-muted">Organization not selected.</div>;
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto p-4 text-text">
      <h1 className="mb-4 text-lg font-semibold">
        {query.trim() ? `Search results for "${query}"` : 'Search'}
      </h1>
      {isLoading && <p className="text-text-muted">Searching...</p>}
      {error && <p className="text-danger">{error}</p>}
      {results && (
        <div className="flex flex-col gap-6">
          <ResultSection title="Messages" empty="No matching messages">
            {results.messages.map((message) => (
              <li key={message.id}>
                <Link
                  to={
                    message.conversation_type === 'channel'
                      ? `/org/${organizationId}/channel/${message.conversation_id}?message=${message.id}`
                      : `/org/${organizationId}/dm/${message.conversation_id}?message=${message.id}`
                  }
                  className="block rounded-md p-2 hover:bg-surface-elevated"
                >
                  <div className="text-sm font-semibold text-accent">
                    {message.author_display_name ?? message.author_id}
                  </div>
                  <MessageContent content={message.content} className="text-sm" />
                </Link>
              </li>
            ))}
          </ResultSection>

          <ResultSection title="Channels" empty="No matching channels">
            {results.channels.map((channel) => (
              <li key={channel.id}>
                <Link
                  to={`/org/${organizationId}/channel/${channel.id}`}
                  className="block rounded-md p-2 text-sm hover:bg-surface-elevated"
                >
                  #{channel.name}
                </Link>
              </li>
            ))}
          </ResultSection>

          <ResultSection title="People" empty="No matching people">
            {results.people.map((person) => (
              <li key={person.id} className="rounded-md p-2 text-sm">
                {person.display_name} <span className="text-text-muted">({person.email})</span>
              </li>
            ))}
          </ResultSection>

          <ResultSection title="Files" empty="No matching files">
            {results.files.map((file) => (
              <li key={file.id} className="rounded-md p-2 text-sm">
                {file.file_name}
              </li>
            ))}
          </ResultSection>
        </div>
      )}
    </div>
  );
}
