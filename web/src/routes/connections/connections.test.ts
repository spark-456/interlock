import { describe, expect, it } from 'vitest';
import type { ConnectionSummary, NodeManifest } from '$lib/api/types';
import { appRows, connectHref } from './connections';

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

  it('starts OAuth only for sign-in apps', () => {
    const [slack, , telegram] = appRows(nodes, conns);
    expect(connectHref(slack!)).toBe('/api/oauth/slack/start');
    expect(connectHref(telegram!)).toBeNull();
  });
});
