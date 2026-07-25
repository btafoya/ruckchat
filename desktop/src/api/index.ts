import { DEFAULT_API_URL } from '../config';
import { ApiClient } from './client';
import { AuthApi } from './auth';
import { ChannelsApi } from './channels';
import { DirectMessagesApi } from './directMessages';
import { FilesApi } from './files';
import { MessagesApi } from './messages';
import { OrganizationsApi } from './organizations';
import { OrgAdminApi } from './orgAdmin';
import { ReactionsApi } from './reactions';
import { SearchApi } from './search';
import { ServerAdminApi } from './serverAdmin';
import { SpellingApi } from './spelling';
import { WebPushApi } from './webPush';

export interface RuckChatApi {
  auth: AuthApi;
  organizations: OrganizationsApi;
  channels: ChannelsApi;
  directMessages: DirectMessagesApi;
  messages: MessagesApi;
  reactions: ReactionsApi;
  files: FilesApi;
  webPush: WebPushApi;
  serverAdmin: ServerAdminApi;
  orgAdmin: OrgAdminApi;
  spelling: SpellingApi;
  search: SearchApi;
}

export function createApi(baseUrl = DEFAULT_API_URL): RuckChatApi {
  const client = new ApiClient(baseUrl);
  return {
    auth: new AuthApi(client),
    organizations: new OrganizationsApi(client),
    channels: new ChannelsApi(client),
    directMessages: new DirectMessagesApi(client),
    messages: new MessagesApi(client),
    reactions: new ReactionsApi(client),
    files: new FilesApi(client),
    webPush: new WebPushApi(client),
    serverAdmin: new ServerAdminApi(client),
    orgAdmin: new OrgAdminApi(client),
    spelling: new SpellingApi(client),
    search: new SearchApi(client),
  };
}

export * from './client';
export * from './error';
export * from './types';
export {
  AuthApi,
  ChannelsApi,
  DirectMessagesApi,
  FilesApi,
  MessagesApi,
  OrganizationsApi,
  OrgAdminApi,
  ReactionsApi,
  SearchApi,
  ServerAdminApi,
  SpellingApi,
  WebPushApi,
};
