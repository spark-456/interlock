import { describe, expect, it } from 'vitest';
import type { Flow, RunEvent } from '$lib/api/types';
import flowJson from '$fixtures/api/flows/flw_larkspur.json';
import events from '$fixtures/streams/runs/run_chat.json';
import { newTurn, passageFor, reduceTurn, suggestions } from './chat';

const flow = flowJson as unknown as Flow;
const replay = (evs: RunEvent[]) => evs.reduce((t, e) => reduceTurn(t, e, flow), newTurn('Refund?'));

describe('playground turn', () => {
  it('folds a recorded chat run into route, tools, answer, sources and actions', () => {
    const t = replay(events as RunEvent[]);
    expect(t.status).toBe('done');
    expect(t.route).toBe('billing');
    expect(t.tools).toEqual(['lookup_customer', 'list_invoices']);
    expect(t.passages).toHaveLength(3);
    expect(t.answer).toMatch(/30-day refund window/);
    expect(t.by).toEqual({ name: 'Billing agent', ms: 1900 });
    expect(t.sources.map((s) => s.title)).toEqual(['Refund policy 2026.pdf', 'Plans and limits']);
    expect(t.actions.map((a) => a.label)).toEqual(['Open refund request', 'No thanks']);
    expect(t.guardrail).toBeUndefined();
    expect(t.path.map((p) => p.nodeId).slice(0, 3)).toEqual(['n_router', 'n_billing', 'n_guard']);
    expect(passageFor(t, 'src_refund')).toMatchObject({ rank: 1, of: 3, passage: { score: 0.91 } });
    expect(passageFor(t, 'missing')).toBeUndefined();
  });

  it('lists each source document once, even when two passages come from it', () => {
    const doubled = (events as RunEvent[]).map((e) => {
      if (e.type !== 'node.items') return e;
      const data = e.data as { items: { json: { sources?: unknown[] } }[] };
      const json = data.items[0]?.json;
      if (!json?.sources?.length) return e;
      const items = [{ ...data.items[0]!, json: { ...json, sources: [json.sources[0], ...json.sources] } }];
      return { ...e, data: { ...data, items } } as unknown as RunEvent;
    });
    expect(replay(doubled).sources.map((s) => s.title)).toEqual(['Refund policy 2026.pdf', 'Plans and limits']);
  });

  it('notes a guardrail when the check after the answer says no', () => {
    const blocked = (events as RunEvent[]).map((e) =>
      e.nodeId === 'n_guard' && e.type === 'node.items' ? ({ ...e, data: { ...e.data, port: 'no' } } as RunEvent) : e
    );
    expect(replay(blocked).guardrail).toBe('Safety rules');
  });

  it('suggests a generic opener when the app has no chat trigger', () => {
    expect(suggestions(flow)).toEqual(['What can you help me with?']);
  });
});
