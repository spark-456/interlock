// Build state → canvas view models.

import type { NodeManifest } from '$lib/api/types';
import type { BuildState } from './buildState';
import { iconFor, isTone, toneFor } from './categories';
import { columnLabels, type ColumnLabel } from './geometry';
import type { CardView, EdgeView, GroupView } from './types';

export type Catalog = Record<string, Pick<NodeManifest, 'title' | 'icon' | 'app'>>;

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

/** Cards grouped for the list view / mobile (M2), in group order. */
export function groupViews(s: BuildState, cards: CardView[]): GroupView[] {
  return s.flow.groups
    .map((g) => {
      const members = cards.filter((c) => c.group === g.id).sort((a, b) => a.position.x - b.position.x || a.position.y - b.position.y);
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
