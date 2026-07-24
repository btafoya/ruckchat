import { ApiClient } from './client';
import type {
  AuditLogEntry,
  CreateServerUserRequest,
  CreateServerUserResponse,
  EndImpersonateRequest,
  ImpersonateRequest,
  ImpersonateResponse,
  Organization,
  RenameOrganizationRequest,
  ResetPasswordResponse,
  ServerSettings,
  ServerUser,
  UpdateServerSettingsRequest,
  UpdateServerUserRequest,
} from './types';

export class ServerAdminApi {
  constructor(private readonly client: ApiClient) {}

  async listOrganizations(token: string): Promise<Organization[]> {
    const response = await this.client.request<{ items: Organization[] }>(
      '/api/v1/server/organizations',
      { token },
    );
    return response.items;
  }

  async createOrganization(
    token: string,
    request: { name: string; slug: string },
  ): Promise<Organization> {
    return this.client.request<Organization>('/api/v1/server/organizations', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async renameOrganization(
    token: string,
    organizationId: string,
    request: RenameOrganizationRequest,
  ): Promise<Organization> {
    return this.client.request<Organization>(
      `/api/v1/server/organizations/${organizationId}`,
      {
        method: 'PATCH',
        token,
        body: request,
      },
    );
  }

  async deleteOrganization(token: string, organizationId: string): Promise<void> {
    await this.client.request<void>(`/api/v1/server/organizations/${organizationId}`, {
      method: 'DELETE',
      token,
    });
  }

  async createUser(
    token: string,
    request: CreateServerUserRequest,
  ): Promise<CreateServerUserResponse> {
    return this.client.request<CreateServerUserResponse>('/api/v1/server/users', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async listUsers(
    token: string,
    options: { limit?: number; offset?: number; isServerAdmin?: boolean } = {},
  ): Promise<ServerUser[]> {
    const params = new URLSearchParams();
    if (options.limit !== undefined) {
      params.set('limit', String(options.limit));
    }
    if (options.offset !== undefined) {
      params.set('offset', String(options.offset));
    }
    if (options.isServerAdmin !== undefined) {
      params.set('is_server_admin', String(options.isServerAdmin));
    }
    const query = params.toString();
    const response = await this.client.request<{ items: ServerUser[] }>(
      `/api/v1/server/users?${query}`,
      { token },
    );
    return response.items;
  }

  async getUser(token: string, userId: string): Promise<ServerUser> {
    return this.client.request<ServerUser>(`/api/v1/server/users/${userId}`, {
      token,
    });
  }

  async updateUser(
    token: string,
    userId: string,
    request: UpdateServerUserRequest,
  ): Promise<ServerUser> {
    return this.client.request<ServerUser>(`/api/v1/server/users/${userId}`, {
      method: 'PATCH',
      token,
      body: request,
    });
  }

  async resetPassword(
    token: string,
    userId: string,
  ): Promise<ResetPasswordResponse> {
    return this.client.request<ResetPasswordResponse>(
      `/api/v1/server/users/${userId}/reset-password`,
      {
        method: 'POST',
        token,
      },
    );
  }

  async promoteUser(token: string, userId: string): Promise<ServerUser> {
    return this.client.request<ServerUser>(
      `/api/v1/server/users/${userId}/promote`,
      {
        method: 'POST',
        token,
      },
    );
  }

  async demoteUser(token: string, userId: string): Promise<ServerUser> {
    return this.client.request<ServerUser>(
      `/api/v1/server/users/${userId}/demote`,
      {
        method: 'POST',
        token,
      },
    );
  }

  async deactivateUser(token: string, userId: string): Promise<ServerUser> {
    return this.client.request<ServerUser>(
      `/api/v1/server/users/${userId}/deactivate`,
      {
        method: 'POST',
        token,
      },
    );
  }

  async reactivateUser(token: string, userId: string): Promise<ServerUser> {
    return this.client.request<ServerUser>(
      `/api/v1/server/users/${userId}/reactivate`,
      {
        method: 'POST',
        token,
      },
    );
  }

  async listServerAdmins(token: string): Promise<ServerUser[]> {
    const response = await this.client.request<{ items: ServerUser[] }>(
      '/api/v1/server/admins',
      { token },
    );
    return response.items;
  }

  async getSettings(token: string): Promise<ServerSettings> {
    return this.client.request<ServerSettings>('/api/v1/server/settings', {
      token,
    });
  }

  async updateSettings(
    token: string,
    request: UpdateServerSettingsRequest,
  ): Promise<void> {
    await this.client.request<void>('/api/v1/server/settings', {
      method: 'PUT',
      token,
      body: request,
    });
  }

  async getAuditLog(
    token: string,
    options: {
      actorId?: string;
      organizationId?: string;
      action?: string;
      resourceType?: string;
      from?: string;
      to?: string;
      limit?: number;
      offset?: number;
    } = {},
  ): Promise<AuditLogEntry[]> {
    const params = new URLSearchParams();
    if (options.actorId) params.set('actor_id', options.actorId);
    if (options.organizationId) params.set('organization_id', options.organizationId);
    if (options.action) params.set('action', options.action);
    if (options.resourceType) params.set('resource_type', options.resourceType);
    if (options.from) params.set('from', options.from);
    if (options.to) params.set('to', options.to);
    if (options.limit !== undefined) params.set('limit', String(options.limit));
    if (options.offset !== undefined) params.set('offset', String(options.offset));
    const query = params.toString();
    const response = await this.client.request<{ items: AuditLogEntry[] }>(
      `/api/v1/server/audit-log?${query}`,
      { token },
    );
    return response.items;
  }

  async impersonate(
    token: string,
    request: ImpersonateRequest,
  ): Promise<ImpersonateResponse> {
    return this.client.request<ImpersonateResponse>('/api/v1/server/impersonate', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async endImpersonate(
    token: string,
    request: EndImpersonateRequest,
  ): Promise<void> {
    await this.client.request<void>('/api/v1/server/impersonate', {
      method: 'DELETE',
      token,
      body: request,
    });
  }
}
