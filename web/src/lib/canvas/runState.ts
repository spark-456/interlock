// Run events → per-node status and item counts, for the editor's live state (PRD §8.4).

import type { Item, RunEvent } from '$lib/api/types';
import type { CardStatus } from './types';

export interface RunState {
  status: 'idle' | 'running' | 'succeeded' | 'failed' | 'cancelled';
  /** Per node; `error` is the failure text for a failed step. */
  nodes: Record<string, { status: CardStatus; label?: string; error?: string }>;
  /** Items out of `node.port`, keyed `nodeId:port`. */
  counts: Record<string, number>;
  /** The first few of those items, for previews and the field mapper. */
  items: Record<string, Item[]>;
}

export const idleRun = (): RunState => ({ status: 'idle', nodes: {}, counts: {}, items: {} });

/** Folds one event into the state. Unknown event types are ignored. A run starting keeps what
 * earlier runs showed, so running one step doesn't blank the rest; callers reset for a full run. */
export function reduceRun(s: RunState, e: RunEvent): RunState {
  const node = (status: CardStatus, label?: string, error?: string): RunState =>
    e.nodeId ? { ...s, nodes: { ...s.nodes, [e.nodeId]: { status, label, error } } } : s;
  switch (e.type) {
    case 'run.started': return { ...s, status: 'running' };
    case 'node.started': return node('building', 'Running');
    case 'node.items': {
      const key = `${e.nodeId}:${e.data.port}`;
      return { ...s, counts: { ...s.counts, [key]: e.data.total }, items: { ...s.items, [key]: e.data.items } };
    }
    case 'node.succeeded': return node('ready', 'Done');
    case 'node.failed': return node('failed', 'Failed', e.data.error);
    case 'node.waiting': return node('warning', 'Waiting for approval');
    case 'approval.decided': return node('building', 'Running');
    case 'run.succeeded': return { ...s, status: 'succeeded' };
    case 'run.failed': return { ...s, status: 'failed' };
    case 'run.cancelled': return { ...s, status: 'cancelled' };
    default: return s;
  }
}
