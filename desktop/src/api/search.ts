import { ApiClient } from './client';
import type { SearchResponse } from './types';

export class SearchApi {
  constructor(private readonly client: ApiClient) {}

  async search(
    token: string,
    organizationId: string,
    query: string,
    limit = 50,
    offset = 0,
  ): Promise<SearchResponse> {
    const params = new URLSearchParams();
    params.set('q', query);
    params.set('limit', String(limit));
    params.set('offset', String(offset));
    return this.client.request<SearchResponse>(
      `/organizations/${organizationId}/search?${params.toString()}`,
      { token },
    );
  }
}
