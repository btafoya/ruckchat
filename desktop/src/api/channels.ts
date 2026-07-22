import { ApiClient } from './client';
import type { Channel, Message, MessageList, PostChannelMessageRequest, UpdateChannelRequest } from './types';

export class ChannelsApi {
  constructor(private readonly client: ApiClient) {}

  async get(token: string, channelId: string): Promise<Channel> {
    return this.client.request<Channel>(`/channels/${channelId}`, {
      token,
    });
  }

  async update(token: string, channelId: string, request: UpdateChannelRequest): Promise<Channel> {
    return this.client.request<Channel>(`/channels/${channelId}`, {
      method: 'PATCH',
      token,
      body: request,
    });
  }

  async archive(token: string, channelId: string): Promise<Channel> {
    return this.client.request<Channel>(`/channels/${channelId}`, {
      method: 'DELETE',
      token,
    });
  }

  async listMessages(
    token: string,
    channelId: string,
    limit = 50,
    offset = 0,
  ): Promise<Message[]> {
    const params = new URLSearchParams();
    params.set('limit', String(limit));
    params.set('offset', String(offset));
    const response = await this.client.request<MessageList>(
      `/channels/${channelId}/messages?${params.toString()}`,
      {
        token,
      },
    );
    return response.items;
  }

  async listReplies(token: string, messageId: string, limit = 50, offset = 0): Promise<Message[]> {
    const params = new URLSearchParams();
    params.set('limit', String(limit));
    params.set('offset', String(offset));
    const response = await this.client.request<MessageList>(
      `/messages/${messageId}/replies?${params.toString()}`,
      {
        token,
      },
    );
    return response.items;
  }

  async postMessage(
    token: string,
    channelId: string,
    request: PostChannelMessageRequest,
  ): Promise<Message> {
    return this.client.request<Message>(`/channels/${channelId}/messages`, {
      method: 'POST',
      token,
      body: request,
    });
  }
}
