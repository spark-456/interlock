import { describe, expect, it } from 'vitest';
import type { ConnectionSummary, FlowSummary } from '$lib/api';
import { checklist, focusFlow } from './checklist';

const flow = (id: string, patch: Partial<FlowSummary> = {}): FlowSummary => ({
  id, name: id, kind: 'automation', version: 1, active: false, updatedAt: '2026-10-30T10:00:00Z', lastRun: null, apps: [], ...patch
});
const run = { id: 'run_1', status: 'succeeded', startedAt: '2026-10-30T10:00:00Z', ms: 900, costUsd: 0, replay: false } as const;
const conn = (status: ConnectionSummary['status']): ConnectionSummary => ({ id: null, app: 'google', label: 'Google', status, account: null });
const done = (flows: FlowSummary[], connections: ConnectionSummary[] = []) =>
  Object.fromEntries(checklist(flows, connections).map((s) => [s.id, s.done]));

describe('checklist', () => {
  it('starts with nothing ticked and every step pointing somewhere useful', () => {
    const steps = checklist([], [conn('notConnected')]);
    expect(steps.map((s) => s.id)).toEqual(['connect', 'build', 'test', 'live']);
    expect(steps.every((s) => !s.done)).toBe(true);
    expect(steps.map((s) => s.href)).toEqual(['/connections', '/new', '/new', '/new']);
  });

  it('ticks connect only for a connected app', () => {
    expect(done([], [conn('expired')]).connect).toBe(false);
    expect(done([], [conn('notConnected'), conn('connected')]).connect).toBe(true);
  });

  it('ticks build, test and live from the flows', () => {
    expect(done([flow('a')])).toMatchObject({ build: true, test: false, live: false });
    expect(done([flow('a'), flow('b', { lastRun: run })])).toMatchObject({ test: true, live: false });
    expect(done([flow('a', { active: true })])).toMatchObject({ test: false, live: true });
  });

  it('sends an AI app to the playground and an automation to the editor', () => {
    const app = checklist([flow('auto'), flow('app', { kind: 'app' })], []);
    expect(app.find((s) => s.id === 'test')?.href).toBe('/flows/app/chat');
    expect(app.find((s) => s.id === 'live')?.href).toBe('/flows/app/publish');
    const auto = checklist([flow('auto')], []);
    expect(auto.find((s) => s.id === 'test')?.href).toBe('/flows/auto');
  });
});

describe('focusFlow', () => {
  it('prefers the first AI app, then the first flow', () => {
    expect(focusFlow([])).toBeNull();
    expect(focusFlow([flow('a'), flow('b')])?.id).toBe('a');
    expect(focusFlow([flow('a'), flow('b', { kind: 'app' }), flow('c', { kind: 'app' })])?.id).toBe('b');
  });
});
