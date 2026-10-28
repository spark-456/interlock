import { describe, expect, it } from 'vitest';
import type { Flow, Item } from '$lib/api/types';
import { previewsFor } from '$lib/previews';
import { chipsOf, sources, upstream } from './mapper';
import { branchOf, emptyOf, fieldsOf, kindOf, problems, readJson, type Schema } from './schema';

const settings: Schema = {
  type: 'object',
  required: ['instructions', 'count'],
  properties: {
    instructions: { type: 'string', title: 'Instructions', 'x-widget': 'longtext' },
    input: { type: 'string', default: '{{ item }}', 'x-widget': 'expression' },
    tone: { type: 'string', enum: ['neutral', 'upbeat'] },
    model: { type: 'string', enum: ['agent', 'fast'], 'x-widget': 'model-role' },
    count: { type: 'integer', minimum: 1, maximum: 5 },
    draft: { type: 'boolean' },
    channels: { type: 'array', items: { type: 'string', enum: ['inapp', 'slack'] } },
    tags: { type: 'array', items: { type: 'string' } },
    cases: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' } } } },
    target: { type: 'object', required: ['url'], properties: { url: { type: 'string' } } },
    auth: {
      oneOf: [
        { title: 'None', properties: {} },
        { title: 'Token', required: ['token'], properties: { token: { type: 'string' } } }
      ]
    }
  }
};

describe('schema', () => {
  it('picks a kind per property', () => {
    expect(fieldsOf(settings).map((f) => [f.key, f.kind])).toEqual([
      ['instructions', 'longtext'], ['input', 'expression'], ['tone', 'select'], ['model', 'model-role'], ['count', 'number'],
      ['draft', 'boolean'], ['channels', 'multiselect'], ['tags', 'list'], ['cases', 'objects'], ['target', 'object'], ['auth', 'oneOf']
    ]);
    expect(kindOf({ type: 'string' })).toBe('text');
    expect(kindOf({ type: 'array', items: { type: 'object' }, 'x-widget': 'json' })).toBe('json');
    expect(fieldsOf(settings).find((f) => f.key === 'instructions')).toMatchObject({ label: 'Instructions', required: true });
  });

  it('reads typed JSON, keeping the draft until it is valid', () => {
    expect(readJson('[{"n": 1}]')).toEqual({ value: [{ n: 1 }] });
    expect(readJson('  ')).toEqual({ value: [] });
    expect(readJson('[{"n": 1},]')).toHaveProperty('error');
  });

  it('starts from defaults or empty values', () => {
    expect(emptyOf(settings)).toMatchObject({
      instructions: '', input: '{{ item }}', count: null, draft: false, channels: [], cases: [], target: { url: '' }
    });
  });

  it('finds the oneOf branch a value belongs to', () => {
    const auth = settings.properties!.auth!;
    expect(branchOf(auth, {})).toBe(0);
    expect(branchOf(auth, { token: 'x' })).toBe(1);
    expect(emptyOf(auth.oneOf![1]!)).toEqual({ token: '' });
  });

  it('reports missing and out-of-range values by path', () => {
    expect(problems(settings, { count: 9, target: {} })).toEqual({
      instructions: 'Instructions is required', count: 'At most 5', 'target.url': 'url is required'
    });
    expect(problems(settings, { instructions: 'Hi', count: 2.5, target: { url: 'u' } })).toEqual({ count: 'A whole number' });
  });
});

const node = (id: string) => ({
  id, type: 'ai.write', typeVersion: 1, name: `Step ${id}`, position: { x: 0, y: 0 }, group: null, settings: {},
  credentials: {}, disabled: false, notes: null
});
const flow = {
  nodes: ['a', 'b', 'c', 'd'].map(node),
  edges: [
    { id: 'e1', from: { node: 'a', port: 'out' }, to: { node: 'b', port: 'in' } },
    { id: 'e2', from: { node: 'b', port: 'out' }, to: { node: 'c', port: 'in' } },
    { id: 'e3', from: { node: 'd', port: 'out' }, to: { node: 'c', port: 'in' } }
  ]
} as unknown as Flow;
const item = (json: Item['json']): Item => ({ json, files: {} });

describe('mapper', () => {
  it('lists every earlier step, nearest first', () => {
    expect(upstream(flow, 'c')).toEqual(['b', 'd', 'a']);
    expect(upstream(flow, 'a')).toEqual([]);
  });

  it('offers the current item and earlier outputs from run samples', () => {
    const s = sources(flow, 'c', { 'b:out': [item({ title: 'Tote', meta: { sku: 'T1' } })] });
    expect(s.slice(0, 3).map((x) => x.expr)).toEqual(['{{ item.title }}', '{{ item.meta }}', '{{ item.meta.sku }}']);
    expect(s).toContainEqual({ expr: '{{ nodes.b.json.title }}', from: 'Step b', field: 'title' });
    expect(s).toContainEqual({ expr: '{{ nodes.a.json }}', from: 'Step a', field: 'whole output' });
  });

  it('falls back to the whole item without a sample', () => {
    expect(sources(flow, 'b', {})[0]).toEqual({ expr: '{{ item }}', from: 'Current item', field: 'whole item' });
  });

  it('reads references as chips', () => {
    expect(chipsOf('{{ item }} {{ item.title }} by {{ nodes.a.json.author }} {{ nodes.b.json }}', flow)).toEqual([
      'Item', 'Item › title', 'Step a › author', 'Step b › output'
    ]);
  });
});

describe('previews', () => {
  const labels = (items: Item[]) => previewsFor(items).map((p) => p.label);
  const file = (mime: string) => ({ id: 'f1', name: 'clip', mime, bytes: 1, previewUrl: null });

  it('ranks the best renderer first and always offers JSON', () => {
    expect(labels([item({ text: 'Hello' })])).toEqual(['Text', 'JSON']);
    expect(labels([item({ subject: 'Hi', body: 'There' })])[0]).toBe('Email');
    expect(labels([item({ a: 1 }), item({ a: 2 })])).toEqual(['Table', 'JSON']);
    expect(labels([{ json: {}, files: { video: file('video/mp4') } } as unknown as Item])[0]).toBe('Videos');
    expect(labels([item([1, 2])])).toEqual(['JSON']);
  });
});
