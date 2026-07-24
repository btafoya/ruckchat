import { ApiClient } from './client';
import type {
  Channel,
  CreateChannelRequest,
  CreateOrganizationRequest,
  Organization,
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
}
