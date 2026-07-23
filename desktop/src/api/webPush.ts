import type { ApiClient } from './client';

export interface VapidKeyResponse {
  public_key: string;
}

export interface SubscribeRequest {
  endpoint: string;
  p256dh: string;
  auth: string;
}

export interface UnsubscribeRequest {
  endpoint: string;
}

export class WebPushApi {
  constructor(private readonly client: ApiClient) {}

  async getVapidKey(): Promise<VapidKeyResponse> {
    return this.client.request<VapidKeyResponse>('/web-push/vapid-key');
  }

  async subscribe(token: string, request: SubscribeRequest): Promise<void> {
    await this.client.request('/web-push/subscribe', {
      method: 'POST',
      body: request,
      token,
    });
  }

  async unsubscribe(token: string, request: UnsubscribeRequest): Promise<void> {
    await this.client.request('/web-push/unsubscribe', {
      method: 'POST',
      body: request,
      token,
    });
  }
}
