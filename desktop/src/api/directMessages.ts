import { ApiClient } from './client';
import type { DirectMessageConversation, Message, MessageList, PostDmMessageRequest, StartDmRequest } from './types';

export class DirectMessagesApi {
  constructor(private readonly client: ApiClient) {}

  async list(token: string): Promise<DirectMessageConversation[]> {
    const response = await this.client.request<{ items: DirectMessageConversation[] }>('/direct_messages', {
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

  async listMessages(token: string, conversationId: string): Promise<Message[]> {
    const response = await this.client.request<MessageList>(
      `/direct_messages/${conversationId}/messages`,
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
