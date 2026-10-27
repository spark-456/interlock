// Field mapper (ED-4): what a step can reference from earlier steps, and how references read.

import type { Flow, Item, JsonValue } from '$lib/api/types';

export interface Source {
  /** Expression inserted into the field, e.g. `{{ nodes.n_products.json.title }}`. */
  expr: string;
  /** Who it comes from ("Current item", or the step's name). */
  from: string;
  /** Field path shown on the chip. */
  field: string;
}

/** Steps that run before `nodeId` (every ancestor along edges), nearest first. */
export function upstream(flow: Flow, nodeId: string): string[] {
  const seen: string[] = [];
  let frontier = [nodeId];
  while (frontier.length) {
    const next = flow.edges.filter((e) => frontier.includes(e.to.node)).map((e) => e.from.node);
    frontier = [...new Set(next)].filter((id) => id !== nodeId && !seen.includes(id));
    seen.push(...frontier);
  }
  return seen;
}

/** Top-level and one-level-nested field paths of an item's JSON. */
function paths(json: JsonValue, prefix = '', depth = 0): string[] {
  if (!json || typeof json !== 'object' || Array.isArray(json)) return [];
  return Object.entries(json).flatMap(([k, v]) => {
    const p = prefix ? `${prefix}.${k}` : k;
    return depth < 1 && v && typeof v === 'object' && !Array.isArray(v) ? [p, ...paths(v, p, depth + 1)] : [p];
  });
}

/** What the step can map: the current item's fields, then each earlier step's output fields.
 * Fields come from the last run's sample items; a step without a sample offers its whole output. */
export function sources(flow: Flow, nodeId: string, samples: Record<string, Item[]>): Source[] {
  const name = (id: string) => flow.nodes.find((n) => n.id === id)?.name ?? id;
  const sampleOf = (id: string) => Object.entries(samples).find(([k, v]) => k.startsWith(`${id}:`) && v.length)?.[1]?.[0];
  const parents = flow.edges.filter((e) => e.to.node === nodeId).map((e) => e.from.node);
  const input = parents.map(sampleOf).find(Boolean);
  const out: Source[] = paths(input?.json ?? null).map((p) => ({ expr: `{{ item.${p} }}`, from: 'Current item', field: p }));
  if (!input) out.push({ expr: '{{ item }}', from: 'Current item', field: 'whole item' });
  for (const id of upstream(flow, nodeId)) {
    const fields = paths(sampleOf(id)?.json ?? null);
    if (fields.length) out.push(...fields.map((p) => ({ expr: `{{ nodes.${id}.json.${p} }}`, from: name(id), field: p })));
    else out.push({ expr: `{{ nodes.${id}.json }}`, from: name(id), field: 'whole output' });
  }
  return out;
}

const REF = /\{\{\s*(.+?)\s*\}\}/g;

/** Readable chips for the references inside a value, e.g. "Three newest products › title". */
export function chipsOf(value: string, flow: Flow): string[] {
  return [...value.matchAll(REF)].map(([, body]) => {
    const m = body!.match(/^nodes\.([\w-]+)\.json\.?(.*)$/);
    if (m) return `${flow.nodes.find((n) => n.id === m[1])?.name ?? m[1]} › ${m[2] || 'output'}`;
    return body!.startsWith('item.') ? `Item › ${body!.slice(5)}` : body!;
  });
}
