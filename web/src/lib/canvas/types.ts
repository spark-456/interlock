// View models the canvas renders. Screens map flows + events to these; the canvas never reads
// API types directly, so the editor (W02), run view (W05) and Building screen share it.

import type { BuildNodeStatus, NodeStat } from '$lib/api/types';
import type { Tone } from './categories';

export type CardStatus = BuildNodeStatus;

export interface CardView {
  id: string;
  title: string;
  /** Type label under the title ("Knowledge base", "Agent"). */
  subtitle: string;
  icon: string;
  tone: Tone;
  stats: NodeStat[];
  status: CardStatus;
  /** Pill text when it differs from the default for the status. */
  label?: string;
  /** 0–1 for the top progress bar while building. */
  progress?: number;
  /** One-line summary for the list view. */
  summary?: string;
  group?: string | null;
  position: { x: number; y: number };
  /** Show the in/out handle dots (sources have no in, outputs no out). */
  showIn: boolean;
  showOut: boolean;
  /** App logo slot key for app nodes. */
  app?: string | null;
}

export type EdgeState = 'ready' | 'queued' | 'active' | 'failed';

export interface EdgeView {
  id: string;
  source: string;
  target: string;
  state: EdgeState;
  /** Vertical link (bottom → top), e.g. the test suite under an agent. */
  vertical?: boolean;
}

export interface GroupView {
  id: string;
  label: string;
  tone: Tone;
  cards: CardView[];
}

/** Default pill text for a status. */
export function pillLabel(card: Pick<CardView, 'status' | 'label'>): string {
  if (card.label) return card.label;
  switch (card.status) {
    case 'ready': return 'Ready';
    case 'waiting': return 'Waiting';
    case 'building': return 'Building';
    case 'fixed': return 'Fixed';
    case 'warning': return 'Needs attention';
    case 'failed': return 'Failed · Retry';
  }
}
