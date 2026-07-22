import { DEFAULT_API_URL } from '../config';
import { ApiClient } from './client';
import { AuthApi } from './auth';
import { ChannelsApi } from './channels';
import { DirectMessagesApi } from './directMessages';
import { FilesApi } from './files';
import { OrganizationsApi } from './organizations';
import { ReactionsApi } from './reactions';

export interface RuckChatApi {
  auth: AuthApi;
  organizations: OrganizationsApi;
  channels: ChannelsApi;
  directMessages: DirectMessagesApi;
  reactions: ReactionsApi;
  files: FilesApi;
}

export function createApi(baseUrl = DEFAULT_API_URL): RuckChatApi {
  const client = new ApiClient(baseUrl);
  return {
    auth: new AuthApi(client),
    organizations: new OrganizationsApi(client),
    channels: new ChannelsApi(client),
    directMessages: new DirectMessagesApi(client),
    reactions: new ReactionsApi(client),
    files: new FilesApi(client),
  };
}

export * from './client';
export * from './error';
export * from './types';
export { AuthApi, ChannelsApi, DirectMessagesApi, FilesApi, OrganizationsApi, ReactionsApi };
