import { ApiClient } from './client';
import type { Message } from './types';

export class MessagesApi {
  constructor(private readonly client: ApiClient) {}

  async edit(token: string, messageId: string, content: string): Promise<Message> {
    return this.client.request<Message>(`/messages/${messageId}`, {
      method: 'PATCH',
      token,
      body: { content },
    });
  }

  async delete(token: string, messageId: string): Promise<void> {
    return this.client.request(`/messages/${messageId}`, {
      method: 'DELETE',
      token,
    });
  }
}
