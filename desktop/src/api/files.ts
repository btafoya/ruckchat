import { ApiClient } from './client';
import type { AttachFileRequest, FileResponse, RecordUploadRequest } from './types';

export class FilesApi {
  constructor(private readonly client: ApiClient) {}

  async recordUpload(token: string, request: RecordUploadRequest): Promise<FileResponse> {
    return this.client.request<FileResponse>('/files/record', {
      method: 'POST',
      token,
      body: request,
    });
  }

  async uploadFile(token: string, organizationId: string, file: File): Promise<FileResponse> {
    const formData = new FormData();
    formData.append('organization_id', organizationId);
    formData.append('file', file);

    return this.client.request<FileResponse>('/files', {
      method: 'POST',
      token,
      formData,
    });
  }

  async attachToMessage(token: string, messageId: string, fileId: string): Promise<void> {
    const request: AttachFileRequest = { file_id: fileId };
    await this.client.request<void>(`/messages/${messageId}/attachments`, {
      method: 'POST',
      token,
      body: request,
    });
  }
}
