import { ApiError } from './error';
import type { ErrorBody } from './types';

export type HttpMethod = 'GET' | 'POST' | 'PATCH' | 'DELETE';

export interface RequestOptions {
  method?: HttpMethod;
  body?: unknown;
  token?: string;
}

function buildHeaders(token: string | undefined): HeadersInit {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    Accept: 'application/json',
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  return headers;
}

async function parseErrorBody(response: Response): Promise<ErrorBody | undefined> {
  const contentType = response.headers.get('content-type') ?? '';
  if (!contentType.includes('application/json')) {
    return undefined;
  }
  try {
    const body = (await response.json()) as unknown;
    if (
      body !== null &&
      typeof body === 'object' &&
      'code' in body &&
      typeof body.code === 'string' &&
      'error' in body &&
      typeof body.error === 'string'
    ) {
      return body as ErrorBody;
    }
  } catch {
    // ignore parsing failures and fall back to status-based message
  }
  return undefined;
}

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async request<T>(path: string, options: RequestOptions = {}): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const init: RequestInit = {
      method: options.method ?? 'GET',
      headers: buildHeaders(options.token),
      credentials: 'include',
    };
    if (options.body !== undefined) {
      init.body = JSON.stringify(options.body);
    }

    const response = await fetch(url, init);
    if (!response.ok) {
      throw new ApiError(response.status, await parseErrorBody(response));
    }

    if (response.status === 204) {
      return undefined as T;
    }

    const contentType = response.headers.get('content-type') ?? '';
    if (!contentType.includes('application/json')) {
      return undefined as T;
    }

    return (await response.json()) as T;
  }
}
