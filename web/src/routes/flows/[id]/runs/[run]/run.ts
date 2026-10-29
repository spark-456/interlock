// Run events → what the run view lists beside the canvas: steps, media clips and the approval.

import type { Decision, RunEvent } from '$lib/api/types';

export type StepStatus = 'running' | 'waiting' | 'succeeded' | 'failed';

export interface Step {
  nodeId: string;
  status: StepStatus;
  count?: number;
  ms?: number;
  costUsd?: number;
  error?: string;
}

/** A video or image a step produced. */
export interface Clip {
  key: string;
  nodeId: string;
  title: string;
  url: string;
  video: boolean;
}

export interface RunView {
  status: 'running' | 'waiting' | 'succeeded' | 'failed' | 'capped';
  trigger?: string;
  startedAt?: string;
  ms?: number;
  costUsd?: number;
  error?: string;
  /** Steps in the order they started. */
  steps: Step[];
  clips: Clip[];
  approval?: { id: string; nodeId: string; decision?: Decision; by?: string | null };
}

export const emptyView = (): RunView => ({ status: 'running', steps: [], clips: [] });

/** Folds one run event into the view. */
export function reduceView(v: RunView, e: RunEvent): RunView {
  const id = e.nodeId ?? '';
  const step = (patch: Partial<Step>): RunView => {
    const old = v.steps.find((s) => s.nodeId === id);
    const next: Step = { nodeId: id, status: 'running', ...old, ...patch };
    return { ...v, steps: old ? v.steps.map((s) => (s === old ? next : s)) : [...v.steps, next] };
  };
  switch (e.type) {
    case 'run.started':
      return { ...v, status: 'running', trigger: e.data.trigger, startedAt: e.ts };
    case 'node.started':
      return step({ status: 'running' });
    case 'node.succeeded':
      return step({ status: 'succeeded', count: e.data.count, ms: e.data.ms, costUsd: e.data.costUsd });
    case 'node.failed':
      return step({ status: 'failed', error: e.data.error });
    case 'node.waiting':
      return { ...step({ status: 'waiting' }), status: 'waiting', approval: { id: e.data.approvalId, nodeId: id } };
    case 'approval.decided':
      return { ...v, status: 'running', approval: { id: e.data.approvalId, nodeId: id, decision: e.data.decision, by: e.data.by } };
    case 'node.items': {
      const clips = e.data.items.flatMap((item, i) =>
        Object.entries(item.files ?? {}).flatMap(([name, f]) => {
          const url = f?.previewUrl ?? (f ? `/api/files/${f.id}` : '');
          const video = !!f?.mime.startsWith('video/');
          if (!f || !(video || f.mime.startsWith('image/'))) return [];
          const t = (item.json as { title?: unknown } | null)?.title;
          const title = typeof t === 'string' ? t : f.name;
          return [{ key: `${id}:${e.data.port}:${i}:${name}`, nodeId: id, title, url, video }];
        })
      );
      return clips.length ? { ...v, clips: [...v.clips.filter((c) => !clips.some((n) => n.key === c.key)), ...clips] } : v;
    }
    case 'run.succeeded':
      return { ...v, status: 'succeeded', ms: e.data.ms, costUsd: e.data.costUsd };
    case 'run.failed':
      return { ...v, status: 'failed', error: e.data.error };
    case 'run.cap_reached':
      return { ...v, status: 'capped', error: `Spending cap reached (${e.data.cap})` };
    default:
      return v;
  }
}
