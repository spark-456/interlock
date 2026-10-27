// Typed API client. All server calls go through here (AGENTS.md §7).
// Fixture mode (VITE_FIXTURES=1) serves every call from web/fixtures/ with no backend.

import type { ApiError } from './types';
import { fixtureRequest, isFixtureMode } from './fixtures';

/** Error thrown for non-2xx responses, carrying the PRD §8.5 error body. */
export class ApiRequestError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
    readonly details: unknown = {}
  ) {
    super(message);
  }
}

type Method = 'GET' | 'POST' | 'PUT' | 'DELETE';

/** Low-level request. Prefer the typed helpers in `api`. */
export async function request<T>(method: Method, path: string, body?: unknown): Promise<T> {
  if (isFixtureMode()) return fixtureRequest<T>(method, path, body);
  const res = await fetch(path, {
    method,
    headers: body === undefined ? {} : { 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
    credentials: 'same-origin'
  });
  if (!res.ok) {
    let err: ApiError | null = null;
    try {
      err = (await res.json()) as ApiError;
    } catch {
      /* not JSON */
    }
    throw new ApiRequestError(
      res.status,
      err?.error.code ?? 'http_error',
      err?.error.message ?? `Request failed (${res.status}).`,
      err?.error.details
    );
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const get = <T>(path: string) => request<T>('GET', path);
export const post = <T>(path: string, body?: unknown) => request<T>('POST', path, body ?? {});
export const put = <T>(path: string, body: unknown) => request<T>('PUT', path, body);
