import { describe, expect, it } from 'vitest';
import type { Flow, FlowSummary, RunSummary } from '$lib/api/types';
import { ago, filterFlows, perDay, sparkPoints, thumbnail } from './flows';

const summary = (id: string, name: string, active: boolean, apps: string[] = []): FlowSummary => ({
  id, name, active, apps, kind: 'automation', version: 1, updatedAt: '2026-10-27T00:00:00Z', lastRun: null
});
const run = (startedAt: string): RunSummary => ({ id: startedAt, status: 'succeeded', startedAt, ms: 1, costUsd: 0, replay: false });
const now = new Date('2026-10-27T12:00:00Z');

describe('flows list', () => {
  const flows = [summary('a', 'Weekly promos', true, ['shopify']), summary('b', 'Support inbox', false, ['gmail'])];

  it('filters by status and searches names and apps', () => {
    expect(filterFlows(flows, 'all', '').map((f) => f.id)).toEqual(['a', 'b']);
    expect(filterFlows(flows, 'live', '').map((f) => f.id)).toEqual(['a']);
    expect(filterFlows(flows, 'off', '').map((f) => f.id)).toEqual(['b']);
    expect(filterFlows(flows, 'all', 'SUPPORT').map((f) => f.id)).toEqual(['b']);
    expect(filterFlows(flows, 'all', 'shop').map((f) => f.id)).toEqual(['a']);
  });

  it('counts runs per day, oldest first, ignoring older runs', () => {
    const runs = [run('2026-10-27T08:00:00Z'), run('2026-10-27T01:00:00Z'), run('2026-10-25T23:59:00Z'), run('2026-09-01T00:00:00Z')];
    expect(perDay(runs, 4, now)).toEqual([0, 1, 0, 2]);
    expect(sparkPoints([0, 2], 10, 10)).toBe('0.0,9.0 10.0,1.0');
  });

  it('scales node positions into the thumbnail and links edges', () => {
    const node = (id: string, x: number, y: number) => ({
      id, type: 'ai.write', typeVersion: 1, name: id, position: { x, y }, group: null, settings: {}, credentials: {}, disabled: false, notes: null
    });
    const flow = {
      nodes: [node('a', 0, 0), node('b', 300, 200)],
      edges: [{ id: 'e', from: { node: 'a', port: 'out' }, to: { node: 'b', port: 'in' } }]
    } as unknown as Flow;
    const t = thumbnail(flow, 300, 112);
    expect(t.nodes[0]).toMatchObject({ x: 8, y: 8 });
    expect(t.nodes[1]!.x).toBeLessThanOrEqual(300 - 8 - 28);
    expect(t.nodes[1]!.y).toBeLessThanOrEqual(112 - 8 - 10);
    expect(t.edges).toHaveLength(1);
  });

  it('formats relative times', () => {
    expect(ago('2026-10-27T11:59:30Z', now)).toBe('just now');
    expect(ago('2026-10-27T11:30:00Z', now)).toBe('30 min ago');
    expect(ago('2026-10-27T02:00:00Z', now)).toBe('10 h ago');
    expect(ago('2026-10-26T06:00:00Z', now)).toBe('yesterday');
    expect(ago('2026-10-20T06:00:00Z', now)).toBe('20 Oct');
  });
});
