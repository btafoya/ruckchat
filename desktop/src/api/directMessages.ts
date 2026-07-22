import { ApiClient } from './client';
import type { DirectMessageConversation, Message, MessageList, PostDmMessageRequest, StartDmRequest } from './types';

export class DirectMessagesApi {
  constructor(private readonly client: ApiClient) {}

  async list(token: string, organizationId: string): Promise<DirectMessageConversation[]> {
    const params = new URLSearchParams();
    params.set('organization_id', organizationId);
    const response = await this.client.request<{ items: DirectMessageConversation[] }>(`/direct_messages?${params.toString()}`, {
      token,
    });
    return response.items;
  }

  async start(token: string, request: StartDmRequest): Promise<DirectMessageConversation> {
    return this.client.request<DirectMessageConversation>('/direct_messages', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async listMessages(
    token: string,
    conversationId: string,
    limit = 50,
    offset = 0,
  ): Promise<Message[]> {
    const params = new URLSearchParams();
    params.set('limit', String(limit));
    params.set('offset', String(offset));
    const response = await this.client.request<MessageList>(
      `/direct_messages/${conversationId}/messages?${params.toString()}`,
      {
        token,
      },
    );
    return response.items;
  }

  async postMessage(
    token: string,
    conversationId: string,
    request: PostDmMessageRequest,
  ): Promise<Message> {
    return this.client.request<Message>(`/direct_messages/${conversationId}/messages`, {
      method: 'POST',
      token,
      body: request,
    });
  }
}
