// Run events → per-node status and item counts, for the editor's live state (PRD §8.4).

import type { RunEvent } from '$lib/api/types';
import type { CardStatus } from './types';

export interface RunState {
  status: 'idle' | 'running' | 'succeeded' | 'failed' | 'cancelled';
  nodes: Record<string, { status: CardStatus; label?: string }>;
  /** Items out of `node.port`, keyed `nodeId:port`. */
  counts: Record<string, number>;
}

export const idleRun = (): RunState => ({ status: 'idle', nodes: {}, counts: {} });

/** Folds one event into the state. Unknown event types are ignored. */
export function reduceRun(s: RunState, e: RunEvent): RunState {
  const node = (status: CardStatus, label?: string): RunState =>
    e.nodeId ? { ...s, nodes: { ...s.nodes, [e.nodeId]: { status, label } } } : s;
  switch (e.type) {
    case 'run.started': return { ...idleRun(), status: 'running' };
    case 'node.started': return node('building', 'Running');
    case 'node.items': return { ...s, counts: { ...s.counts, [`${e.nodeId}:${e.data.port}`]: e.data.total } };
    case 'node.succeeded': return node('ready', 'Done');
    case 'node.failed': return node('failed', 'Failed');
    case 'node.waiting': return node('warning', 'Waiting for approval');
    case 'approval.decided': return node('building', 'Running');
    case 'run.succeeded': return { ...s, status: 'succeeded' };
    case 'run.failed': return { ...s, status: 'failed' };
    case 'run.cancelled': return { ...s, status: 'cancelled' };
    default: return s;
  }
}
