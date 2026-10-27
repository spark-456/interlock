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
  return parse<T>(
    await fetch(path, {
      method,
      headers: body === undefined ? {} : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
      credentials: 'same-origin'
    })
  );
}

/** Multipart upload (`POST /api/files`). Fixture mode echoes each file back as a `FileRef`. */
export async function upload<T>(path: string, files: File[]): Promise<T> {
  if (isFixtureMode()) {
    const refs = files.map((f, i) => ({ id: `file_fx${i}`, mime: f.type, bytes: f.size, name: f.name, previewUrl: null }));
    return refs as T;
  }
  const form = new FormData();
  for (const f of files) form.append('file', f, f.name);
  return parse<T>(await fetch(path, { method: 'POST', body: form, credentials: 'same-origin' }));
}

async function parse<T>(res: Response): Promise<T> {
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
export const del = <T>(path: string) => request<T>('DELETE', path);
