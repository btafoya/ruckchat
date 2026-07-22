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

  async listMessages(token: string, channelId: string): Promise<Message[]> {
    const response = await this.client.request<MessageList>(`/channels/${channelId}/messages`, {
      token,
    });
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
