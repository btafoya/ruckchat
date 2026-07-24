import { ApiClient } from './client';
import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
  RegistrationStatusResponse,
  UpdateProfileRequest,
  User,
} from './types';

export class AuthApi {
  constructor(private readonly client: ApiClient) {}

  async register(request: RegisterRequest): Promise<RegisterResponse> {
    return this.client.request<RegisterResponse>('/auth/register', {
      method: 'POST',
      body: request,
    });
  }

  async getRegistrationStatus(): Promise<RegistrationStatusResponse> {
    return this.client.request<RegistrationStatusResponse>(
      '/auth/registration-status',
    );
  }

  async login(request: LoginRequest): Promise<LoginResponse> {
    return this.client.request<LoginResponse>('/auth/login', {
      method: 'POST',
      body: request,
    });
  }

  async logout(token: string): Promise<void> {
    await this.client.request<void>('/auth/logout', {
      method: 'POST',
      token,
    });
  }

  async getProfile(token: string): Promise<User> {
    return this.client.request<User>('/users/me', {
      token,
    });
  }

  async updateProfile(token: string, request: UpdateProfileRequest): Promise<User> {
    return this.client.request<User>('/users/me', {
      method: 'PATCH',
      token,
      body: request,
    });
  }
}
