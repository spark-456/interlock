import { describe, expect, it } from 'vitest';
import type { Flow, NodeManifest, RunEvent } from '$lib/api/types';
import {
  COLUMN, addNode, connect, diffFlows, edgeNear, insertOnEdge, record, redo, removeNodes, replaceNode, tidy, undo, type Manifest
} from './edit';
import { paletteGroups } from './palette';
import { idleRun, reduceRun } from './runState';
import { flowCards, flowEdges, readiness } from './views';

const node = (id: string, type: string, x: number, settings: object = {}) => ({
  id, type, typeVersion: 1, name: id, position: { x, y: 0 }, group: null, settings, credentials: {}, disabled: false, notes: null
});
const link = (id: string, from: string, to: string, port = 'out') => ({ id, from: { node: from, port }, to: { node: to, port: 'in' } });

function sample(): Flow {
  return {
    id: 'f', workspaceId: 'w', version: 1, name: 'Sample', kind: 'automation',
    nodes: [node('a', 'trigger.manual', 0), node('b', 'ai.write', 304, { tone: 'upbeat', length: 'short' })],
    edges: [link('e1', 'a', 'b')],
    groups: [],
    settings: { timezone: 'UTC', spendCapUsdPerRun: null },
    meta: { createdBy: null, goal: null }
  } as unknown as Flow;
}

const manifest = (typeId: string, props: object = {}, outputs = ['out']): Manifest => ({
  typeId, version: 1, title: typeId, inputs: [{ name: 'in', label: null }],
  outputs: outputs.map((name) => ({ name, label: null })), settingsSchema: { properties: props }
});
const h = () => 160;

describe('editing', () => {
  it('adds a node with schema defaults and a fresh id', () => {
    const r = addNode(sample(), manifest('ai.agent', { model: { default: 'agent' } }), { x: 9, y: 9 });
    expect(r.id).toBe('n_1');
    expect(r.flow.nodes.at(-1)).toMatchObject({ type: 'ai.agent', settings: { model: 'agent' }, position: { x: 9, y: 9 } });
  });

  it('inserts on an edge between its two steps', () => {
    const r = insertOnEdge(sample(), 'e1', manifest('logic.if'), { x: 150, y: 0 })!;
    expect(r.flow.edges.map((e) => `${e.from.node}>${e.to.node}`)).toEqual(['a>n_1', 'n_1>b']);
    expect(insertOnEdge(sample(), 'e1', { ...manifest('trigger.x'), inputs: [] }, { x: 0, y: 0 })).toBeNull();
  });

  it('replaces a type in place, keeping shared settings and remapping ports', () => {
    const f = { ...sample(), edges: [...sample().edges, link('e2', 'b', 'a', 'out')] };
    const r = replaceNode(f, 'b', manifest('logic.if', { tone: {}, cond: { default: true } }, ['true', 'false']));
    const b = r.nodes.find((n) => n.id === 'b')!;
    expect(b).toMatchObject({ type: 'logic.if', position: { x: 304, y: 0 }, settings: { tone: 'upbeat', cond: true } });
    expect(r.edges.find((e) => e.id === 'e2')!.from.port).toBe('true');
  });

  it('removes nodes with their edges and ignores duplicate or self connections', () => {
    expect(removeNodes(sample(), ['b']).edges).toEqual([]);
    expect(connect(sample(), { node: 'a', port: 'out' }, { node: 'b', port: 'in' }).edges).toHaveLength(1);
    expect(connect(sample(), { node: 'a', port: 'out' }, { node: 'a', port: 'in' }).edges).toHaveLength(1);
  });

  it('tidies into columns by longest path', () => {
    const f = { ...sample(), nodes: [...sample().nodes, node('c', 'ai.write', 5)], edges: [link('e1', 'a', 'b'), link('e2', 'b', 'c'), link('e3', 'a', 'c')] };
    expect(tidy(f, h).nodes.map((n) => n.position.x)).toEqual([0, COLUMN, 2 * COLUMN]);
  });

  it('diffs added, changed and removed steps, ignoring moves', () => {
    const before = sample();
    const after = { ...before, nodes: [{ ...before.nodes[0]!, position: { x: 99, y: 99 } }, node('c', 'ai.write', 600)], edges: [link('e9', 'a', 'c')] };
    after.nodes[0] = { ...after.nodes[0]!, name: 'Start' };
    const d = diffFlows(before, after);
    expect([...d.added]).toEqual(['c']);
    expect([...d.changed]).toEqual(['a']);
    expect(d.removed.map((n) => n.id)).toEqual(['b']);
    expect([...d.addedEdges]).toEqual(['e9']);
    expect(d.removedEdges.map((e) => e.id)).toEqual(['e1']);
  });

  it('finds the edge under a point', () => {
    expect(edgeNear(sample(), { x: 284, y: 80 }, h)).toBe('e1');
    expect(edgeNear(sample(), { x: 284, y: 400 }, h)).toBeNull();
  });

  it('undoes and redoes', () => {
    const a = sample();
    const b = removeNodes(a, ['b']);
    const back = undo(record({ past: [], future: [] }, a), b)!;
    expect(back.flow).toBe(a);
    expect(redo(back.history, back.flow)!.flow).toBe(b);
    expect(undo({ past: [], future: [] }, a)).toBeNull();
  });
});

describe('views', () => {
  const ev = (seq: number, type: string, nodeId: string | null, data: object = {}) => ({ seq, ts: '', runId: 'r', nodeId, type, data }) as unknown as RunEvent;

  it('shows run state on cards and item counts on edges', () => {
    let run = idleRun();
    for (const e of [ev(1, 'run.started', null), ev(2, 'node.succeeded', 'a'), ev(3, 'node.items', 'a', { port: 'out', total: 3 }), ev(4, 'node.started', 'b')]) {
      run = reduceRun(run, e);
    }
    const cards = flowCards(sample(), {}, run);
    expect(cards.map((c) => [c.id, c.status])).toEqual([['a', 'ready'], ['b', 'building']]);
    expect(flowEdges(sample(), run)[0]).toMatchObject({ state: 'active', count: 3 });
    run = reduceRun(run, ev(5, 'node.failed', 'b', { error: 'missing credential `google`', retryable: false }));
    expect(run.nodes.b).toEqual({ status: 'failed', label: 'Failed', error: 'missing credential `google`' });
  });

  it('marks a proposal and keeps removed steps visible', () => {
    const after = { ...sample(), nodes: [sample().nodes[0]!, node('c', 'ai.write', 600)], edges: [] };
    const d = diffFlows(sample(), after);
    expect(flowCards(after, {}, idleRun(), d).map((c) => [c.id, c.diff, c.label])).toEqual([
      ['a', undefined, undefined], ['c', 'added', 'Added'], ['b', 'removed', 'Removed']
    ]);
    expect(flowEdges(after, idleRun(), d)).toMatchObject([{ id: 'removed-e1', state: 'failed', diff: 'removed' }]);
  });

  it('turns settings into short card rows', () => {
    const [, b] = flowCards(sample(), { 'ai.write': { title: 'Write', icon: 'pen', app: null, settingsSchema: { properties: { tone: { title: 'Tone' } } } } }, idleRun());
    expect(b!.stats).toEqual([{ label: 'Tone', value: 'upbeat' }, { label: 'Length', value: 'short' }]);
  });
});

describe('palette', () => {
  it('searches and groups the catalog', () => {
    const m = (typeId: string, category: string, title: string) => ({ typeId, category, title, description: '' }) as NodeManifest;
    const nodes = [m('ai.write', 'ai', 'Write'), m('trigger.manual', 'trigger', 'Manual'), m('app.slack.send', 'app', 'Slack message')];
    expect(paletteGroups(nodes, '').map((g) => g.label)).toEqual(['Triggers', 'AI', 'Apps']);
    expect(paletteGroups(nodes, 'slack')).toEqual([{ label: 'Apps', items: [nodes[2]] }]);
  });
});

describe('step readiness', () => {
  it('flags a missing connection, then missing settings, before any run', () => {
    const node = { ...sample().nodes[1]!, credentials: {}, settings: {} };
    const spec = { key: 'discord', app: 'discord', kind: 'webhook' as const, label: 'Discord webhook', scopes: [], optional: false };
    expect(readiness(node, { title: 'Send', icon: 'send', app: 'discord', credentials: [spec] })).toEqual({ status: 'warning', label: 'Connect Discord' });
    const schema = { type: 'object', required: ['text'], properties: { text: { type: 'string', title: 'Text' } } };
    expect(readiness(node, { title: 'Send', icon: 'send', app: null, settingsSchema: schema })).toEqual({ status: 'warning', label: 'Needs settings' });
    expect(readiness({ ...node, settings: { text: 'hi' } }, { title: 'Send', icon: 'send', app: null, settingsSchema: schema })).toEqual({ status: 'ready' });
  });
});
