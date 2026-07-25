import { ApiClient } from './client';
import type {
  Channel,
  ChannelMembership,
  Message,
  MessagePageQuery,
  MessagePageResponse,
  PostChannelMessageRequest,
  UpdateChannelRequest,
} from './types';
import { messagePageParams } from './types';

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

  async unarchive(token: string, channelId: string): Promise<Channel> {
    return this.client.request<Channel>(`/channels/${channelId}/unarchive`, {
      method: 'POST',
      token,
    });
  }

  async listMembers(token: string, channelId: string): Promise<ChannelMembership[]> {
    const response = await this.client.request<{ items: ChannelMembership[] }>(
      `/channels/${channelId}/members`,
      { token },
    );
    return response.items;
  }

  async addMember(token: string, channelId: string, userId: string): Promise<ChannelMembership> {
    return this.client.request<ChannelMembership>(
      `/channels/${channelId}/members?user_id=${encodeURIComponent(userId)}`,
      {
        method: 'POST',
        token,
      },
    );
  }

  async removeMember(token: string, channelId: string, userId: string): Promise<void> {
    await this.client.request<void>(
      `/channels/${channelId}/members?user_id=${encodeURIComponent(userId)}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async listMessages(
    token: string,
    channelId: string,
    query: MessagePageQuery = {},
  ): Promise<MessagePageResponse> {
    const params = messagePageParams(query);
    return this.client.request<MessagePageResponse>(
      `/channels/${channelId}/messages?${params.toString()}`,
      {
        token,
      },
    );
  }

  async listReplies(
    token: string,
    messageId: string,
    query: MessagePageQuery = {},
  ): Promise<MessagePageResponse> {
    const params = messagePageParams(query);
    return this.client.request<MessagePageResponse>(
      `/messages/${messageId}/replies?${params.toString()}`,
      {
        token,
      },
    );
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

  async markRead(token: string, channelId: string, messageIds: string[]): Promise<void> {
    await this.client.request<void>(`/channels/${channelId}/read`, {
      method: 'POST',
      token,
      body: { message_ids: messageIds },
    });
  }
}
