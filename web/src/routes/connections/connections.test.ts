import { describe, expect, it } from 'vitest';
import type { ConnectionSummary, NodeManifest } from '$lib/api/types';
import { appRows, connectHref, keyFields, takesKey, type AppRow } from './connections';

const node = (...specs: [string, string, string][]) =>
  ({ credentials: specs.map(([app, kind, label]) => ({ key: app, app, kind, label, scopes: [], optional: false })) }) as unknown as NodeManifest;

describe('connections', () => {
  const nodes = [
    node(['google', 'oAuth2', 'Google account'], ['http', 'apiKey', 'API key or token']),
    node(['telegram', 'botToken', 'Telegram bot token'], ['google', 'oAuth2', 'Google account']),
    node(['slack', 'oAuth2', 'Slack workspace'])
  ];
  const conns: ConnectionSummary[] = [
    { id: 'c1', app: 'slack', label: 'Slack', status: 'connected', account: 'Larkspur' },
    { id: 'c2', app: 'unknown', label: 'Gone', status: 'connected', account: null }
  ];

  it('lists each catalog app once, connected first, without per-step keys', () => {
    const rows = appRows(nodes, conns);
    expect(rows.map((r) => [r.name, r.status])).toEqual([
      ['Slack', 'connected'],
      ['Google', 'notConnected'],
      ['Telegram', 'notConnected']
    ]);
    expect(rows[0]?.account).toBe('Larkspur');
  });

  it('starts OAuth only for apps with a sign-in, and takes a bot token for Slack', () => {
    const [slack, google, telegram] = appRows(nodes, conns);
    expect(connectHref(google!)).toBe('/api/oauth/google/start');
    expect(connectHref(slack!)).toBeNull();
    expect(connectHref(telegram!)).toBeNull();
    expect(keyFields(slack!).map((f) => f.key)).toEqual(['bot_token']);
    expect([slack, telegram].every((r) => takesKey(r!))).toBe(true);
    expect(takesKey({ ...slack!, app: 'linkedin' })).toBe(false);
  });

  it('asks key-based apps for the fields their steps read', () => {
    const row = (app: string, kind: AppRow['kind']): AppRow => ({ app, name: app, label: 'Key', kind, status: 'notConnected', account: null });
    expect(keyFields(row('bluesky', 'appPassword')).map((f) => f.key)).toEqual(['identifier', 'app_password']);
    expect(keyFields(row('telegram', 'botToken')).map((f) => f.key)).toEqual(['token']);
    expect(keyFields(row('acme', 'webhook')).map((f) => f.key)).toEqual(['url']);
    expect(keyFields(row('acme', 'apiKey'))[0]).toMatchObject({ key: 'api_key', secret: true });
  });
});
