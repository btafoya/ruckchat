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
export type RegistrationStatusResponse =
  components['schemas']['RegistrationStatusResponse'];
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

export type ServerUser = components['schemas']['ServerUserResponse'];
export type ServerUserList = components['schemas']['ServerUserList'];
export type CreateServerUserRequest = components['schemas']['CreateServerUserRequest'];
export type CreateServerUserResponse = components['schemas']['CreateServerUserResponse'];
export type ServerSettings = components['schemas']['ServerSettings'];
export type UpdateServerSettingsRequest =
  components['schemas']['UpdateServerSettingsRequest'];
export type AuditLogEntry = components['schemas']['AuditLogEntry'];
export type AuditLogList = components['schemas']['AuditLogList'];
export type RenameOrganizationRequest =
  components['schemas']['RenameOrganizationRequest'];
export type UpdateServerUserRequest =
  components['schemas']['UpdateServerUserRequest'];
export type ResetPasswordResponse = components['schemas']['ResetPasswordResponse'];
export type ImpersonateRequest = components['schemas']['ImpersonateRequest'];
export type ImpersonateResponse = components['schemas']['ImpersonateResponse'];
export type EndImpersonateRequest = components['schemas']['EndImpersonateRequest'];

export type OrganizationSettings = components['schemas']['OrganizationSettings'];
export type UpdateOrganizationSettingsRequest =
  components['schemas']['UpdateOrganizationSettingsRequest'];
export type Member = components['schemas']['MemberResponse'];
export type MemberList = components['schemas']['MemberList'];
export type OrganizationRole = components['schemas']['OrganizationRole'];
export type Permission = components['schemas']['Permission'];
export type Team = components['schemas']['Team'];
export type CustomEmoji = components['schemas']['CustomEmoji'];
export type CreateRoleRequest = components['schemas']['CreateRoleRequest'];
export type CreatePermissionRequest = components['schemas']['CreatePermissionRequest'];
export type CreateTeamRequest = components['schemas']['CreateTeamRequest'];
export type CreateEmojiRequest = components['schemas']['CreateEmojiRequest'];
export type UpdateRoleRequest = components['schemas']['UpdateRoleRequest'];
export type UpdatePermissionRequest = components['schemas']['UpdatePermissionRequest'];
export type UpdateTeamRequest = components['schemas']['UpdateTeamRequest'];
