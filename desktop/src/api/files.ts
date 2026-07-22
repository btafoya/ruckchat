import { ApiClient } from './client';
import type { AttachFileRequest, FileResponse, RecordUploadRequest } from './types';

export class FilesApi {
  constructor(private readonly client: ApiClient) {}

  async recordUpload(token: string, request: RecordUploadRequest): Promise<FileResponse> {
    return this.client.request<FileResponse>('/files', {
      method: 'POST',
      token,
      body: request,
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
