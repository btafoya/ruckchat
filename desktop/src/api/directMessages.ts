import { ApiClient } from './client';
import type {
  DirectMessageConversation,
  Message,
  MessagePageQuery,
  MessagePageResponse,
  PostDmMessageRequest,
  StartDmRequest,
} from './types';
import { messagePageParams } from './types';

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

  async hide(token: string, conversationId: string): Promise<void> {
    await this.client.request<void>(`/direct_messages/${conversationId}/hide`, {
      method: 'POST',
      token,
    });
  }

  async listMessages(
    token: string,
    conversationId: string,
    query: MessagePageQuery = {},
  ): Promise<MessagePageResponse> {
    const params = messagePageParams(query);
    return this.client.request<MessagePageResponse>(
      `/direct_messages/${conversationId}/messages?${params.toString()}`,
      {
        token,
      },
    );
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

  async markRead(token: string, conversationId: string, messageIds: string[]): Promise<void> {
    await this.client.request<void>(`/direct_messages/${conversationId}/read`, {
      method: 'POST',
      token,
      body: { message_ids: messageIds },
    });
  }
}
