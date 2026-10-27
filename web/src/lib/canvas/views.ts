// Build state, saved flows and runs → canvas view models.

import type { Edge, Flow, Group, JsonValue, Node, NodeManifest, NodeStat } from '$lib/api/types';
import type { BuildState } from './buildState';
import { iconFor, isTone, toneFor } from './categories';
import type { FlowDiff } from './edit';
import { columnLabels, type ColumnLabel } from './geometry';
import type { RunState } from './runState';
import type { CardStatus, CardView, Diff, EdgeView, GroupView } from './types';

export type Catalog = Record<string, Pick<NodeManifest, 'title' | 'icon' | 'app'> & Partial<Pick<NodeManifest, 'settingsSchema'>>>;

/** Cards for every node in the flow being built. */
export function cardViews(s: BuildState, catalog: Catalog = {}): CardView[] {
  return s.flow.nodes.map((n) => {
    const st = s.nodes[n.id] ?? { status: 'waiting' as const, stats: [] };
    const m = catalog[n.type];
    const tone = isTone(st.tone) ? st.tone : toneFor(n.type);
    return {
      id: n.id,
      title: n.name,
      subtitle: st.subtitle ?? m?.title ?? n.type,
      icon: st.icon ?? iconFor(n.type, m?.icon),
      tone,
      stats: st.stats,
      status: st.status,
      label: st.label,
      progress: st.progress,
      summary: st.summary,
      group: n.group,
      position: n.position,
      showIn: tone !== 'source' && tone !== 'trigger',
      showOut: tone !== 'output',
      app: m?.app ?? null
    };
  });
}

/** Edge state from its endpoints: queued until the target starts, active while it builds. */
export function edgeViews(s: BuildState): EdgeView[] {
  const status = (id: string) => s.nodes[id]?.status ?? 'waiting';
  return s.flow.edges.map((e) => {
    const src = status(e.from.node);
    const dst = status(e.to.node);
    const state: EdgeView['state'] =
      dst === 'failed' ? 'failed'
      : dst === 'building' && src !== 'waiting' ? 'active'
      : dst === 'waiting' || src === 'waiting' ? 'queued'
      : 'ready';
    return { id: e.id, source: e.from.node, target: e.to.node, state, vertical: e.from.port === 'bottom' };
  });
}

/** Column labels from the flow's groups. */
export function labelViews(s: BuildState, cards: CardView[]): ColumnLabel[] {
  return columnLabels(cards, s.flow.groups);
}

/** Cards grouped for the list view / mobile (M2), in group order; ungrouped cards come last. */
export function groupViews(groups: Group[], cards: CardView[]): GroupView[] {
  const known = new Set(groups.map((g) => g.id));
  const loose = cards.some((c) => !c.group || !known.has(c.group));
  return [...groups, ...(loose ? [{ id: '', label: 'Other steps' }] : [])]
    .map((g) => {
      const members = cards.filter((c) => (g.id ? c.group === g.id : !c.group || !known.has(c.group))).sort((a, b) => a.position.x - b.position.x || a.position.y - b.position.y);
      return { id: g.id, label: g.label, tone: members[0]?.tone ?? 'logic', cards: members };
    })
    .filter((g) => g.cards.length > 0);
}

/** "Ready", "In progress" or "Next" for a group header. */
export function groupStatus(g: GroupView): string {
  if (g.cards.some((c) => c.status === 'building' || c.status === 'failed')) return 'In progress';
  if (g.cards.every((c) => c.status === 'waiting')) return 'Next';
  if (g.cards.every((c) => c.status !== 'waiting')) return 'Ready';
  return 'In progress';
}

const DIFF_PILL: Record<Diff, { status: CardStatus; label: string }> = {
  added: { status: 'ready', label: 'Added' },
  changed: { status: 'warning', label: 'Changed' },
  removed: { status: 'failed', label: 'Removed' }
};

/** Cards for a saved flow: live run status when a run is on, Ask diff marks when a proposal is shown. */
export function flowCards(flow: Flow, catalog: Catalog, run: RunState, diff?: FlowDiff): CardView[] {
  const card = (n: Node, d?: Diff): CardView => {
    const m = catalog[n.type];
    const tone = toneFor(n.type);
    const live = run.status === 'idle' ? undefined : (run.nodes[n.id] ?? { status: 'waiting' as const, label: 'Not run' });
    const pill = d ? DIFF_PILL[d] : n.disabled ? { status: 'waiting' as const, label: 'Off' } : (live ?? { status: 'ready' as const });
    return {
      id: n.id,
      title: n.name,
      subtitle: m?.title ?? n.type,
      icon: iconFor(n.type, m?.icon),
      tone,
      stats: settingStats(n, m?.settingsSchema),
      status: pill.status,
      label: pill.label,
      group: n.group,
      position: n.position,
      showIn: tone !== 'trigger',
      showOut: tone !== 'output',
      app: m?.app ?? null,
      diff: d
    };
  };
  const mark = (id: string): Diff | undefined => (diff?.added.has(id) ? 'added' : diff?.changed.has(id) ? 'changed' : undefined);
  return [...flow.nodes.map((n) => card(n, mark(n.id))), ...(diff?.removed ?? []).map((n) => card(n, 'removed'))];
}

/** Edges for a saved flow, animated while their target runs, with item counts from the run. */
export function flowEdges(flow: Flow, run: RunState, diff?: FlowDiff): EdgeView[] {
  const status = (id: string) => (run.status === 'idle' ? 'ready' : (run.nodes[id]?.status ?? 'waiting'));
  const view = (e: Edge, d?: Diff): EdgeView => {
    const dst = status(e.to.node);
    const state: EdgeView['state'] =
      d === 'removed' || dst === 'failed' ? 'failed'
      : d === 'added' || dst === 'building' ? 'active'
      : dst === 'waiting' || status(e.from.node) === 'waiting' ? 'queued'
      : 'ready';
    return {
      id: e.id, source: e.from.node, target: e.to.node, state, vertical: e.from.port === 'bottom',
      count: run.counts[`${e.from.node}:${e.from.port}`], diff: d
    };
  };
  return [
    ...flow.edges.map((e) => view(e, diff?.addedEdges.has(e.id) ? 'added' : undefined)),
    ...(diff?.removedEdges ?? []).map((e) => view({ ...e, id: `removed-${e.id}` }, 'removed'))
  ];
}

type Schema = { properties?: Record<string, { title?: string }> };

/** Up to three settings as card rows: short values only, labelled by their schema title. */
function settingStats(n: Node, schema?: JsonValue): NodeStat[] {
  const titles = (schema as Schema | undefined)?.properties ?? {};
  const settings = (n.settings ?? {}) as Record<string, JsonValue>;
  return Object.entries(settings)
    .filter(([, v]) => v !== null && v !== '' && (typeof v !== 'object' || Array.isArray(v)))
    .slice(0, 3)
    .map(([k, v]) => ({ label: titles[k]?.title ?? humanize(k), value: short(v) }));
}

const humanize = (key: string) => key.replace(/[_-]+/g, ' ').replace(/([a-z])([A-Z])/g, '$1 $2').replace(/^./, (c) => c.toUpperCase());

function short(v: JsonValue): string {
  if (Array.isArray(v)) return `${v.length} ${v.length === 1 ? 'item' : 'items'}`;
  if (typeof v === 'boolean') return v ? 'Yes' : 'No';
  const text = String(v);
  if (text.includes('{{')) return 'Mapped';
  return text.length > 18 ? `${text.slice(0, 17)}…` : text;
}
