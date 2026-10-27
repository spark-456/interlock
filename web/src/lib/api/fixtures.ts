// Fixture mode: maps API paths to files under web/fixtures/.
//
//   GET  /api/flows/flw_promos      → fixtures/api/flows/flw_promos.json
//   POST /api/builds                → fixtures/api/builds.post.json
//   SSE  /api/builds/bld_fa/events  → fixtures/streams/builds/bld_fa.json (array of events)
//
// Missing fixtures fall back to `<dir>/_default.json` (e.g. any flow id → flows/_default.json).

import { ApiRequestError } from './client';

const files = import.meta.glob<unknown>('/fixtures/**/*.json', { import: 'default' });

/** True when the app runs without a backend. */
export function isFixtureMode(): boolean {
  return import.meta.env.VITE_FIXTURES === '1';
}

/** Turns an API path + method into candidate fixture keys, most specific first. */
export function fixtureKeys(method: string, path: string): string[] {
  const clean = path.split('?')[0]!.replace(/\/+$/, '').replace(/^\//, '');
  const suffix = method === 'GET' ? '' : `.${method.toLowerCase()}`;
  const keys = [`/fixtures/${clean}${suffix}.json`];
  const parent = clean.split('/').slice(0, -1).join('/');
  if (parent) keys.push(`/fixtures/${parent}/_default${suffix}.json`);
  return keys;
}

/** Stream fixture key for an SSE path (`/api/runs/x/events` → streams/runs/x). */
export function streamKeys(path: string): string[] {
  const m = path.split('?')[0]!.match(/^\/api\/(builds|runs)\/([^/]+)\/events$/);
  if (!m) return [];
  return [`/fixtures/streams/${m[1]}/${m[2]}.json`, `/fixtures/streams/${m[1]}/_default.json`];
}

/** Loads the first fixture that exists among `keys`. */
export async function loadFixture<T>(keys: string[]): Promise<T | undefined> {
  for (const key of keys) {
    const load = files[key];
    if (load) return structuredClone((await load()) as T);
  }
  return undefined;
}

/** Serves a request from fixtures, with a small delay so loading states are visible. */
export async function fixtureRequest<T>(method: string, path: string, _body?: unknown): Promise<T> {
  const data = await loadFixture<T>(fixtureKeys(method, path));
  await new Promise((r) => setTimeout(r, 60));
  if (data === undefined) {
    throw new ApiRequestError(404, 'fixture_missing', `No fixture for ${method} ${path}.`);
  }
  return data;
}
