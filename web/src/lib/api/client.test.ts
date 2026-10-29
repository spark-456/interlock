import { afterEach, describe, expect, it, vi } from 'vitest';
import { get, post } from './client';

const reply = (status: number, body: string, type?: string) =>
  vi.stubGlobal('fetch', async () => new Response(body || null, { status, headers: type ? { 'content-type': type } : {} }));

afterEach(() => vi.unstubAllGlobals());

describe('client responses', () => {
  it('accepts a 202 with no body (approval decisions, build answers)', async () => {
    reply(202, '');
    await expect(post('/api/approvals/apr_1/decide', { decision: 'approve' })).resolves.toBeUndefined();
  });

  it('returns plain-text bodies as text and JSON bodies parsed', async () => {
    reply(200, '# Guide', 'text/markdown; charset=utf-8');
    await expect(get('/api/flows/flw_1/guide')).resolves.toBe('# Guide');
    reply(200, '{"ok":true}', 'application/json');
    await expect(get('/healthz')).resolves.toEqual({ ok: true });
  });
});
