// Editor operations on a flow (ED-1, ED-2, ED-6, ED-9). Pure: each returns a new flow.

import type { Edge, Flow, JsonValue, Node, NodeManifest, PortRef } from '$lib/api/types';
import { NODE_WIDTH } from './geometry';

export type Manifest = Pick<NodeManifest, 'typeId' | 'version' | 'title' | 'inputs' | 'outputs' | 'settingsSchema'>;
type Point = { x: number; y: number };
type Schema = { properties?: Record<string, { default?: JsonValue }> };

/** Column step used by tidy: card width plus a 40 px gutter (the builder's layout). */
export const COLUMN = NODE_WIDTH + 40;
const ROW_GAP = 32;

function nextId(prefix: string, taken: Iterable<string>): string {
  const used = new Set(taken);
  let i = 1;
  while (used.has(`${prefix}_${i}`)) i++;
  return `${prefix}_${i}`;
}

const props = (m: Manifest) => (m.settingsSchema as Schema | null)?.properties ?? {};
const inPort = (m: Manifest) => m.inputs[0]?.name;
const outPort = (m: Manifest) => m.outputs[0]?.name;

function defaults(m: Manifest): Record<string, JsonValue> {
  const out: Record<string, JsonValue> = {};
  for (const [key, p] of Object.entries(props(m))) if (p.default !== undefined) out[key] = p.default;
  return out;
}

function edge(flow: Flow, from: PortRef, to: PortRef, taken: Edge[] = flow.edges): Edge {
  return { id: nextId('e', taken.map((e) => e.id)), from, to };
}

/** Adds a node of type `m` at `at`; returns the new flow and the node id. */
export function addNode(flow: Flow, m: Manifest, at: Point): { flow: Flow; id: string } {
  const id = nextId('n', flow.nodes.map((n) => n.id));
  const node: Node = {
    id, type: m.typeId, typeVersion: m.version, name: m.title, position: { ...at }, group: null,
    settings: defaults(m), credentials: {}, disabled: false, notes: null
  };
  return { flow: { ...flow, nodes: [...flow.nodes, node] }, id };
}

/** Drops a new node onto an edge: A → B becomes A → new → B. Null when the type can't sit in between. */
export function insertOnEdge(flow: Flow, edgeId: string, m: Manifest, at: Point): { flow: Flow; id: string } | null {
  const old = flow.edges.find((e) => e.id === edgeId);
  const input = inPort(m);
  const output = outPort(m);
  if (!old || !input || !output) return null;
  const added = addNode(flow, m, at);
  const rest = added.flow.edges.filter((e) => e.id !== edgeId);
  const first = edge(flow, old.from, { node: added.id, port: input }, rest);
  const second = edge(flow, { node: added.id, port: output }, old.to, [...rest, first]);
  return { flow: { ...added.flow, edges: [...rest, first, second] }, id: added.id };
}

/** Swaps a node's type in place: keeps id, name, position, group, settings the new type has, and
 * connections whose ports still exist (others move to the new type's first port, or are dropped). */
export function replaceNode(flow: Flow, id: string, m: Manifest): Flow {
  const known = props(m);
  const outs = m.outputs.map((p) => p.name);
  const ins = m.inputs.map((p) => p.name);
  const nodes = flow.nodes.map((n) => {
    if (n.id !== id) return n;
    const kept = Object.entries((n.settings ?? {}) as Record<string, JsonValue>).filter(([k]) => k in known);
    return { ...n, type: m.typeId, typeVersion: m.version, settings: { ...defaults(m), ...Object.fromEntries(kept) } };
  });
  const remap = (port: string, ports: string[]) => (ports.includes(port) ? port : ports[0]);
  const edges = flow.edges.flatMap((e) => {
    const from = e.from.node === id ? remap(e.from.port, outs) : e.from.port;
    const to = e.to.node === id ? remap(e.to.port, ins) : e.to.port;
    return from && to ? [{ ...e, from: { ...e.from, port: from }, to: { ...e.to, port: to } }] : [];
  });
  return { ...flow, nodes, edges };
}

/** Removes nodes and every edge touching them. */
export function removeNodes(flow: Flow, ids: string[]): Flow {
  const gone = new Set(ids);
  return {
    ...flow,
    nodes: flow.nodes.filter((n) => !gone.has(n.id)),
    edges: flow.edges.filter((e) => !gone.has(e.from.node) && !gone.has(e.to.node))
  };
}

/** Connects two ports, ignoring self-links and duplicates. */
export function connect(flow: Flow, from: PortRef, to: PortRef): Flow {
  if (from.node === to.node) return flow;
  const same = (a: PortRef, b: PortRef) => a.node === b.node && a.port === b.port;
  if (flow.edges.some((e) => same(e.from, from) && same(e.to, to))) return flow;
  return { ...flow, edges: [...flow.edges, edge(flow, from, to)] };
}

/** Moves nodes to new positions. */
export function moveNodes(flow: Flow, moves: Record<string, Point>): Flow {
  return { ...flow, nodes: flow.nodes.map((n) => (moves[n.id] ? { ...n, position: { ...moves[n.id]! } } : n)) };
}

/** Left-to-right layout: column = longest path from a start node; rows keep their current order. */
export function tidy(flow: Flow, height: (id: string) => number): Flow {
  const depth = new Map<string, number>(flow.nodes.map((n) => [n.id, 0]));
  // Longest-path relaxation; capped at the node count so cycles can't loop forever.
  for (let pass = 0; pass < flow.nodes.length; pass++) {
    let moved = false;
    for (const e of flow.edges) {
      const d = (depth.get(e.from.node) ?? 0) + 1;
      if (depth.has(e.to.node) && d > depth.get(e.to.node)!) {
        depth.set(e.to.node, d);
        moved = true;
      }
    }
    if (!moved) break;
  }
  const columns = new Map<number, Node[]>();
  for (const n of flow.nodes) columns.set(depth.get(n.id)!, [...(columns.get(depth.get(n.id)!) ?? []), n]);
  const moves: Record<string, Point> = {};
  for (const [col, nodes] of columns) {
    let y = 0;
    for (const n of [...nodes].sort((a, b) => a.position.y - b.position.y)) {
      moves[n.id] = { x: col * COLUMN, y };
      y += height(n.id) + ROW_GAP;
    }
  }
  return moveNodes(flow, moves);
}

export interface FlowDiff {
  added: Set<string>;
  changed: Set<string>;
  /** Nodes only in `before`, to draw as removed. */
  removed: Node[];
  addedEdges: Set<string>;
  removedEdges: Edge[];
}

/** What a proposed flow adds, changes and removes. Moving a node alone is not a change. */
export function diffFlows(before: Flow, after: Flow): FlowDiff {
  const old = new Map(before.nodes.map((n) => [n.id, n]));
  const next = new Set(after.nodes.map((n) => n.id));
  const content = (n: Node) => JSON.stringify({ ...n, position: null });
  const edgeKey = (e: Edge) => `${e.from.node}.${e.from.port}>${e.to.node}.${e.to.port}`;
  const oldEdges = new Set(before.edges.map(edgeKey));
  const newEdges = new Set(after.edges.map(edgeKey));
  return {
    added: new Set(after.nodes.filter((n) => !old.has(n.id)).map((n) => n.id)),
    changed: new Set(after.nodes.filter((n) => old.has(n.id) && content(old.get(n.id)!) !== content(n)).map((n) => n.id)),
    removed: before.nodes.filter((n) => !next.has(n.id)),
    addedEdges: new Set(after.edges.filter((e) => !oldEdges.has(edgeKey(e))).map((e) => e.id)),
    removedEdges: before.edges.filter((e) => !newEdges.has(edgeKey(e)))
  };
}

/** The edge whose curve passes within `radius` of `p` (flow coordinates), if any. */
export function edgeNear(flow: Flow, p: Point, height: (id: string) => number, radius = 24): string | null {
  const nodes = new Map(flow.nodes.map((n) => [n.id, n]));
  let best: { id: string; d: number } | null = null;
  for (const e of flow.edges) {
    const a = nodes.get(e.from.node);
    const b = nodes.get(e.to.node);
    if (!a || !b) continue;
    const s = { x: a.position.x + NODE_WIDTH, y: a.position.y + height(a.id) / 2 };
    const t = { x: b.position.x, y: b.position.y + height(b.id) / 2 };
    const k = Math.max(48, Math.abs(t.x - s.x) / 2);
    for (let i = 0; i <= 24; i++) {
      const u = i / 24;
      const x = cubic(s.x, s.x + k, t.x - k, t.x, u);
      const y = cubic(s.y, s.y, t.y, t.y, u);
      const d = Math.hypot(x - p.x, y - p.y);
      if (d <= radius && (!best || d < best.d)) best = { id: e.id, d };
    }
  }
  return best?.id ?? null;
}

function cubic(a: number, b: number, c: number, d: number, u: number): number {
  const v = 1 - u;
  return v * v * v * a + 3 * v * v * u * b + 3 * v * u * u * c + u * u * u * d;
}

/** Undo/redo stacks of whole flows. */
export interface History {
  past: Flow[];
  future: Flow[];
}

const LIMIT = 50;

/** Records `before` so the change that follows can be undone. */
export function record(h: History, before: Flow): History {
  return { past: [...h.past.slice(-(LIMIT - 1)), before], future: [] };
}

/** Steps back: returns the flow to show and the new stacks, or null when there's nothing to undo. */
export function undo(h: History, current: Flow): { flow: Flow; history: History } | null {
  const flow = h.past.at(-1);
  return flow ? { flow, history: { past: h.past.slice(0, -1), future: [current, ...h.future] } } : null;
}

/** Steps forward again after an undo. */
export function redo(h: History, current: Flow): { flow: Flow; history: History } | null {
  const flow = h.future[0];
  return flow ? { flow, history: { past: [...h.past, current], future: h.future.slice(1) } } : null;
}
