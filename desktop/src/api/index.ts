import { DEFAULT_API_URL } from '../config';
import { ApiClient } from './client';
import { AuthApi } from './auth';
import { ChannelsApi } from './channels';
import { DirectMessagesApi } from './directMessages';
import { OrganizationsApi } from './organizations';

export interface RuckChatApi {
  auth: AuthApi;
  organizations: OrganizationsApi;
  channels: ChannelsApi;
  directMessages: DirectMessagesApi;
}

export function createApi(baseUrl = DEFAULT_API_URL): RuckChatApi {
  const client = new ApiClient(baseUrl);
  return {
    auth: new AuthApi(client),
    organizations: new OrganizationsApi(client),
    channels: new ChannelsApi(client),
    directMessages: new DirectMessagesApi(client),
  };
}

export * from './client';
export * from './error';
export * from './types';
export { AuthApi, ChannelsApi, DirectMessagesApi, OrganizationsApi };
