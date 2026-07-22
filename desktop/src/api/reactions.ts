import { ApiClient } from './client';
import type { AddReactionRequest, Reaction } from './types';

export class ReactionsApi {
  constructor(private readonly client: ApiClient) {}

  async add(token: string, messageId: string, emoji: string): Promise<Reaction> {
    const request: AddReactionRequest = { emoji };
    return this.client.request<Reaction>(`/messages/${messageId}/reactions`, {
      method: 'POST',
      token,
      body: request,
    });
  }

  async remove(token: string, messageId: string, emoji: string): Promise<void> {
    const encodedEmoji = encodeURIComponent(emoji);
    await this.client.request<void>(`/messages/${messageId}/reactions/${encodedEmoji}`, {
      method: 'DELETE',
      token,
    });
  }
}
