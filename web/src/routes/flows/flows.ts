import type { Flow, FlowSummary, RunSummary } from '$lib/api/types';

export type Filter = 'all' | 'live' | 'off';

/** Flows matching the filter chip and the search text (name or app). */
export function filterFlows(flows: FlowSummary[], filter: Filter, query: string): FlowSummary[] {
  const q = query.trim().toLowerCase();
  return flows.filter(
    (f) =>
      (filter === 'all' || f.active === (filter === 'live')) &&
      (!q || f.name.toLowerCase().includes(q) || f.apps.some((a) => a.includes(q)))
  );
}

/** Runs per day for the last `days` days, oldest first, for the sparkline. */
export function perDay(runs: RunSummary[], days: number, now: Date): number[] {
  const counts = new Array<number>(days).fill(0);
  const today = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
  for (const r of runs) {
    const t = new Date(r.startedAt);
    const age = Math.round((today - Date.UTC(t.getUTCFullYear(), t.getUTCMonth(), t.getUTCDate())) / 86_400_000);
    if (age >= 0 && age < days) counts[days - 1 - age]! += 1;
  }
  return counts;
}

/** SVG polyline points for `values` in a `w`×`h` box. */
export function sparkPoints(values: number[], w: number, h: number): string {
  const max = Math.max(1, ...values);
  const step = values.length > 1 ? w / (values.length - 1) : 0;
  return values.map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * (h - 2) - 1).toFixed(1)}`).join(' ');
}

/** Node boxes scaled into a `w`×`h` thumbnail, keeping the canvas aspect ratio. */
export function thumbnail(flow: Flow, w: number, h: number) {
  const pad = 8;
  const xs = flow.nodes.map((n) => n.position.x);
  const ys = flow.nodes.map((n) => n.position.y);
  const [minX, minY] = [Math.min(...xs), Math.min(...ys)];
  const spanX = Math.max(1, Math.max(...xs) - minX);
  const spanY = Math.max(1, Math.max(...ys) - minY);
  const scale = Math.min((w - 2 * pad - 28) / spanX, (h - 2 * pad - 10) / spanY);
  const at = (id: string) => {
    const n = flow.nodes.find((x) => x.id === id);
    return n ? { x: pad + (n.position.x - minX) * scale, y: pad + (n.position.y - minY) * scale } : undefined;
  };
  const nodes = flow.nodes.map((n) => ({ id: n.id, type: n.type, ...at(n.id)! }));
  const edges = flow.edges.flatMap((e) => {
    const [a, b] = [at(e.from.node), at(e.to.node)];
    return a && b ? [{ id: e.id, x1: a.x + 28, y1: a.y + 5, x2: b.x, y2: b.y + 5 }] : [];
  });
  return { nodes, edges };
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** "3 min ago", "yesterday", "26 Oct". */
export function ago(iso: string, now: Date): string {
  const s = Math.max(0, (now.getTime() - new Date(iso).getTime()) / 1000);
  if (s < 60) return 'just now';
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86_400) return `${Math.floor(s / 3600)} h ago`;
  if (s < 172_800) return 'yesterday';
  // Not toLocaleDateString: its first call can take seconds while the runtime loads locale data.
  const d = new Date(iso);
  return `${d.getDate()} ${MONTHS[d.getMonth()]}`;
}
