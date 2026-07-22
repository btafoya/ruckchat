import type { ErrorBody } from './types';

export class ApiError extends Error {
  readonly code: string;
  readonly status: number;
  readonly body?: ErrorBody;

  constructor(status: number, body?: ErrorBody) {
    super(body?.error ?? `HTTP ${status}`);
    this.code = body?.code ?? 'unknown';
    this.status = status;
    this.body = body;
  }
}

export function isUnauthorizedError(error: unknown): boolean {
  return error instanceof ApiError && error.status === 401;
}
