import { ApiClient } from './client';
import type {
  AddTeamMemberRequest,
  AddTeamRoomRequest,
  CreateEmojiRequest,
  CreatePermissionRequest,
  CreateRoleRequest,
  CreateTeamRequest,
  CustomEmoji,
  Member,
  OrganizationRole,
  OrganizationSettings,
  Permission,
  Team,
  TeamMember,
  TeamMembership,
  TeamRoom,
  UpdateOrganizationSettingsRequest,
  UpdatePermissionRequest,
  UpdateRoleRequest,
  UpdateTeamRequest,
} from './types';

export class OrgAdminApi {
  constructor(private readonly client: ApiClient) {}

  async listMembers(token: string, organizationId: string): Promise<Member[]> {
    const response = await this.client.request<{ items: Member[] }>(
      `/organizations/${organizationId}/members`,
      { token },
    );
    return response.items;
  }

  async getSettings(
    token: string,
    organizationId: string,
  ): Promise<OrganizationSettings> {
    return this.client.request<OrganizationSettings>(
      `/api/v1/admin/organizations/${organizationId}/settings`,
      { token },
    );
  }

  async updateSettings(
    token: string,
    organizationId: string,
    request: UpdateOrganizationSettingsRequest,
  ): Promise<OrganizationSettings> {
    return this.client.request<OrganizationSettings>(
      `/api/v1/admin/organizations/${organizationId}/settings`,
      {
        method: 'PUT',
        token,
        body: request,
      },
    );
  }

  async listRoles(token: string, organizationId: string): Promise<OrganizationRole[]> {
    const response = await this.client.request<{ items: OrganizationRole[] }>(
      `/api/v1/admin/organizations/${organizationId}/roles`,
      { token },
    );
    return response.items;
  }

  async createRole(
    token: string,
    organizationId: string,
    request: CreateRoleRequest,
  ): Promise<OrganizationRole> {
    return this.client.request<OrganizationRole>(
      `/api/v1/admin/organizations/${organizationId}/roles`,
      {
        method: 'POST',
        token,
        body: request,
      },
    );
  }

  async updateRole(
    token: string,
    organizationId: string,
    roleId: string,
    request: UpdateRoleRequest,
  ): Promise<OrganizationRole> {
    return this.client.request<OrganizationRole>(
      `/api/v1/admin/organizations/${organizationId}/roles/${roleId}`,
      {
        method: 'PATCH',
        token,
        body: request,
      },
    );
  }

  async deleteRole(
    token: string,
    organizationId: string,
    roleId: string,
  ): Promise<void> {
    await this.client.request<void>(
      `/api/v1/admin/organizations/${organizationId}/roles/${roleId}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async listPermissions(
    token: string,
    organizationId: string,
  ): Promise<Permission[]> {
    const response = await this.client.request<{ items: Permission[] }>(
      `/api/v1/admin/organizations/${organizationId}/permissions`,
      { token },
    );
    return response.items;
  }

  async createPermission(
    token: string,
    organizationId: string,
    request: CreatePermissionRequest,
  ): Promise<Permission> {
    return this.client.request<Permission>(
      `/api/v1/admin/organizations/${organizationId}/permissions`,
      {
        method: 'POST',
        token,
        body: request,
      },
    );
  }

  async updatePermission(
    token: string,
    organizationId: string,
    permissionId: string,
    request: UpdatePermissionRequest,
  ): Promise<Permission> {
    return this.client.request<Permission>(
      `/api/v1/admin/organizations/${organizationId}/permissions/${permissionId}`,
      {
        method: 'PATCH',
        token,
        body: request,
      },
    );
  }

  async deletePermission(
    token: string,
    organizationId: string,
    permissionId: string,
  ): Promise<void> {
    await this.client.request<void>(
      `/api/v1/admin/organizations/${organizationId}/permissions/${permissionId}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async listEmoji(
    token: string,
    organizationId: string,
  ): Promise<CustomEmoji[]> {
    const response = await this.client.request<{ items: CustomEmoji[] }>(
      `/api/v1/admin/organizations/${organizationId}/emoji`,
      { token },
    );
    return response.items;
  }

  async createEmoji(
    token: string,
    organizationId: string,
    request: CreateEmojiRequest,
  ): Promise<CustomEmoji> {
    return this.client.request<CustomEmoji>(
      `/api/v1/admin/organizations/${organizationId}/emoji`,
      {
        method: 'POST',
        token,
        body: request,
      },
    );
  }

  async deleteEmoji(
    token: string,
    organizationId: string,
    emojiId: string,
  ): Promise<void> {
    await this.client.request<void>(
      `/api/v1/admin/organizations/${organizationId}/emoji/${emojiId}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async listTeams(token: string, organizationId: string): Promise<Team[]> {
    const response = await this.client.request<{ items: Team[] }>(
      `/api/v1/admin/organizations/${organizationId}/teams`,
      { token },
    );
    return response.items;
  }

  async createTeam(
    token: string,
    organizationId: string,
    request: CreateTeamRequest,
  ): Promise<Team> {
    return this.client.request<Team>(
      `/api/v1/admin/organizations/${organizationId}/teams`,
      {
        method: 'POST',
        token,
        body: request,
      },
    );
  }

  async updateTeam(
    token: string,
    organizationId: string,
    teamId: string,
    request: UpdateTeamRequest,
  ): Promise<Team> {
    return this.client.request<Team>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}`,
      {
        method: 'PATCH',
        token,
        body: request,
      },
    );
  }

  async deleteTeam(
    token: string,
    organizationId: string,
    teamId: string,
  ): Promise<void> {
    await this.client.request<void>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async listTeamMembers(
    token: string,
    organizationId: string,
    teamId: string,
  ): Promise<TeamMember[]> {
    const response = await this.client.request<{ items: TeamMember[] }>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}/members`,
      { token },
    );
    return response.items;
  }

  async addTeamMember(
    token: string,
    organizationId: string,
    teamId: string,
    request: AddTeamMemberRequest,
  ): Promise<TeamMembership> {
    return this.client.request<TeamMembership>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}/members`,
      {
        method: 'POST',
        token,
        body: request,
      },
    );
  }

  async removeTeamMember(
    token: string,
    organizationId: string,
    teamId: string,
    userId: string,
  ): Promise<void> {
    await this.client.request<void>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}/members/${userId}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }

  async listTeamRooms(
    token: string,
    organizationId: string,
    teamId: string,
  ): Promise<TeamRoom[]> {
    const response = await this.client.request<{ items: TeamRoom[] }>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}/rooms`,
      { token },
    );
    return response.items;
  }

  async addTeamRoom(
    token: string,
    organizationId: string,
    teamId: string,
    request: AddTeamRoomRequest,
  ): Promise<TeamRoom> {
    return this.client.request<TeamRoom>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}/rooms`,
      {
        method: 'POST',
        token,
        body: request,
      },
    );
  }

  async removeTeamRoom(
    token: string,
    organizationId: string,
    teamId: string,
    channelId: string,
  ): Promise<void> {
    await this.client.request<void>(
      `/api/v1/admin/organizations/${organizationId}/teams/${teamId}/rooms/${channelId}`,
      {
        method: 'DELETE',
        token,
      },
    );
  }
}
