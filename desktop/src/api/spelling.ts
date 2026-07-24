import type { ApiClient } from './client';
import type {
  SpellingCheckRequest,
  SpellingCheckResponse,
  SpellingLanguageList,
  SpellingSuggestRequest,
  SpellingSuggestResponse,
} from './types';

export class SpellingApi {
  constructor(private readonly client: ApiClient) {}

  async check(token: string, request: SpellingCheckRequest): Promise<SpellingCheckResponse> {
    return this.client.request<SpellingCheckResponse>('/api/v1/spelling/check', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async suggest(token: string, request: SpellingSuggestRequest): Promise<SpellingSuggestResponse> {
    return this.client.request<SpellingSuggestResponse>('/api/v1/spelling/suggest', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async languages(token: string): Promise<SpellingLanguageList> {
    return this.client.request<SpellingLanguageList>('/api/v1/spelling/languages', {
      token,
    });
  }
}
