import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import type { BuildEvent } from '$lib/api/types';
import { initialBuildState, progress, reduce, testsRun, timeLeft, type BuildState } from './buildState';
import { applyPatch } from './jsonPatch';
import { cardViews, edgeViews, groupStatus, groupViews, labelViews } from './views';
import { bezier, nodeHeight } from './geometry';

const stream = JSON.parse(
  readFileSync(join(process.cwd(), 'fixtures/streams/builds/bld_larkspur.json'), 'utf8')
) as BuildEvent[];

function replay(untilSec = Infinity): BuildState {
  const t0 = Date.parse(stream[0]!.ts);
  return stream.filter((e) => Date.parse(e.ts) - t0 <= untilSec * 1000).reduce(reduce, initialBuildState());
}

describe('jsonPatch', () => {
  it('adds, replaces and removes', () => {
    const doc = { nodes: [{ id: 'a' }], name: 'x' };
    const out = applyPatch(doc, [
      { op: 'add', path: '/nodes/-', value: { id: 'b' } },
      { op: 'replace', path: '/name', value: 'y' },
      { op: 'remove', path: '/nodes/0' },
      { op: 'add', path: '/meta/goal', value: 'g' }
    ]);
    expect(out).toEqual({ nodes: [{ id: 'b' }], name: 'y', meta: { goal: 'g' } });
    expect(doc.nodes).toHaveLength(1);
  });
});

describe('geometry', () => {
  it('follows the design formulas', () => {
    expect([nodeHeight(1), nodeHeight(2), nodeHeight(3)]).toEqual([138, 160, 182]);
    expect(bezier(0, 0, 40, 10)).toBe('M0 0 C48 0 -8 10 40 10');
    expect(bezier(0, 0, 304, 0)).toBe('M0 0 C152 0 152 0 304 0');
  });
});

describe('build state from the recorded Larkspur build', () => {
  it('matches the design frame at 1:28', () => {
    const s = replay(88);
    expect(s.flow.name).toBe('Larkspur Support');
    expect(s.elapsed).toBe(88_000);
    expect(s.flow.nodes).toHaveLength(13);
    expect(s.nodes['n_api']?.label).toBe('8 endpoints skipped');
    expect(s.nodes['n_out_api']?.status).toBe('waiting');
    expect(s.stages['read']).toMatchObject({ status: 'done', start: 0, end: 24_000 });
    expect(s.stages['test']?.status).toBe('active');
    expect(testsRun(s)).toEqual({ run: 21, total: 30 });
    expect(s.costUsd).toBe(0.061);
    expect(s.log.at(-1)?.text).toBe('Running tests: 21 of 30 done, 19 passed so far');
    expect(progress(s)).toBeCloseTo((5 + 21 / 30) / 7, 5);
    expect(timeLeft(s)).toBe(25_000);
  });

  it('derives canvas views', () => {
    const s = replay(88);
    const cards = cardViews(s);
    const kb = cards.find((c) => c.id === 'n_kb')!;
    expect(kb).toMatchObject({ tone: 'kb', icon: 'db', subtitle: 'Knowledge base', status: 'ready' });
    expect(cards.find((c) => c.id === 'n_web')).toMatchObject({ tone: 'source', showIn: false });
    expect(cards.find((c) => c.id === 'n_out_mcp')).toMatchObject({ tone: 'output', showOut: false });
    const edges = edgeViews(s);
    expect(edges.find((e) => e.id === 'e_guard_out_api')?.state).toBe('queued');
    expect(edges.find((e) => e.id === 'e_billing_tests')).toMatchObject({ state: 'active', vertical: true });
    expect(labelViews(s, cards).map((l) => [l.text, l.x, l.tone])).toEqual([
      ['Inputs', 0, 'source'], ['Understanding', 304, 'kb'], ['Agents', 608, 'agent'], ['Safety', 912, 'guard'], ['Outputs', 1216, 'output']
    ]);
    const groups = groupViews(s.flow.groups, cards);
    expect(groups.map(groupStatus)).toEqual(['Ready', 'Ready', 'In progress', 'Ready', 'Next']);
  });

  it('ends ready at 1:52 with two fixes', () => {
    const s = replay();
    expect(s.status).toBe('ready');
    expect(s.flowId).toBe('flw_larkspur');
    expect(s.elapsed).toBe(112_000);
    expect(s.fixes).toHaveLength(2);
    expect(s.tests).toEqual({ passed: 30, failed: 0 });
    expect(s.costUsd).toBe(0.084);
    expect(progress(s)).toBe(1);
  });

  it('ignores unknown and duplicate events', () => {
    const s = replay(10);
    const same = reduce(s, { ...stream[0]!, seq: s.lastSeq } as BuildEvent);
    expect(same).toBe(s);
    const unknown = reduce(s, { seq: s.lastSeq + 1, ts: stream[0]!.ts, buildId: 'x', nodeId: null, type: 'build.future', data: {} } as unknown as BuildEvent);
    expect(unknown.flow).toBe(s.flow);
  });
});
