import type { components } from './schema';

export type User = components['schemas']['UserResponse'];
export type Organization = components['schemas']['Organization'];
export type Channel = components['schemas']['Channel'];
export type DirectMessageConversation = components['schemas']['DirectMessageConversation'];
export type Message = components['schemas']['Message'];
export type MessageList = components['schemas']['MessageList'];
export type File = components['schemas']['File'];
export type FileResponse = components['schemas']['FileResponse'];
export type Reaction = components['schemas']['Reaction'];
export type ErrorBody = components['schemas']['Error'];
export type Role = components['schemas']['Role'];

export type RegisterRequest = components['schemas']['RegisterRequest'];
export type RegisterResponse = components['schemas']['RegisterResponse'];
export type LoginRequest = components['schemas']['LoginRequest'];
export type LoginResponse = components['schemas']['LoginResponse'];
export type CreateOrganizationRequest = components['schemas']['CreateOrganizationRequest'];
export type CreateChannelRequest = components['schemas']['CreateChannelRequest'];
export type UpdateChannelRequest = components['schemas']['UpdateChannelRequest'];
export type PostChannelMessageRequest = components['schemas']['PostChannelMessageRequest'];
export type PostDmMessageRequest = components['schemas']['PostDmMessageRequest'];
export type StartDmRequest = components['schemas']['StartDmRequest'];
export type UpdateProfileRequest = components['schemas']['UpdateProfileRequest'];
export type AddReactionRequest = components['schemas']['AddReactionRequest'];
export type RecordUploadRequest = components['schemas']['RecordUploadRequest'];
export type AttachFileRequest = components['schemas']['AttachFileRequest'];
