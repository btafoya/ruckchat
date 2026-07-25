import { ApiClient } from './client';
import type {
  ChangeRoleRequest,
  Channel,
  CreateChannelRequest,
  CreateOrganizationRequest,
  InviteMemberRequest,
  Member,
  Organization,
  UnreadCountsResponse,
  User,
} from './types';

export class OrganizationsApi {
  constructor(private readonly client: ApiClient) {}

  async list(token: string): Promise<Organization[]> {
    const response = await this.client.request<{ items: Organization[] }>('/organizations', {
      token,
    });
    return response.items;
  }

  async listMembers(token: string, organizationId: string): Promise<Member[]> {
    const response = await this.client.request<{ items: Member[] }>(
      `/organizations/${organizationId}/members`,
      { token },
    );
    return response.items;
  }

  async inviteMember(
    token: string,
    organizationId: string,
    request: InviteMemberRequest,
  ): Promise<void> {
    await this.client.request<void>(`/organizations/${organizationId}/members`, {
      method: 'POST',
      token,
      body: request,
    });
  }

  async changeRole(
    token: string,
    organizationId: string,
    request: ChangeRoleRequest,
  ): Promise<void> {
    await this.client.request<void>(`/organizations/${organizationId}/members`, {
      method: 'PATCH',
      token,
      body: request,
    });
  }

  async removeMember(token: string, organizationId: string, userId: string): Promise<void> {
    await this.client.request<void>(
      `/organizations/${organizationId}/members?user_id=${encodeURIComponent(userId)}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async create(token: string, request: CreateOrganizationRequest): Promise<Organization> {
    return this.client.request<Organization>('/organizations', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async listChannels(token: string, organizationId: string): Promise<Channel[]> {
    const response = await this.client.request<{ items: Channel[] }>(
      `/organizations/${organizationId}/channels`,
      {
        token,
      },
    );
    return response.items;
  }

  async createChannel(
    token: string,
    organizationId: string,
    request: CreateChannelRequest,
  ): Promise<Channel> {
    return this.client.request<Channel>(`/organizations/${organizationId}/channels`, {
      method: 'POST',
      token,
      body: request,
    });
  }

  async searchMembers(
    token: string,
    organizationId: string,
    query: string,
  ): Promise<User[]> {
    const response = await this.client.request<{ items: User[] }>(
      `/organizations/${organizationId}/members/search?q=${encodeURIComponent(query)}`,
      {
        token,
      },
    );
    return response.items;
  }

  async unreadCounts(token: string, organizationId: string): Promise<UnreadCountsResponse> {
    return this.client.request<UnreadCountsResponse>(
      `/organizations/${organizationId}/unread_counts`,
      { token },
    );
  }
}
