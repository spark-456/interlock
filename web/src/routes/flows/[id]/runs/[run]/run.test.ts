import { describe, expect, it } from 'vitest';
import type { RunEvent } from '$lib/api/types';
import events from '$fixtures/streams/runs/run_fa.json';
import { emptyView, reduceView } from './run';

const replay = (evs: RunEvent[]) => evs.reduce(reduceView, emptyView());

describe('run view', () => {
  it('folds the recorded run into steps, clips and the approval', () => {
    const v = replay(events as RunEvent[]);
    expect(v).toMatchObject({ status: 'succeeded', trigger: 'trigger.schedule', ms: 48200, costUsd: 0.011 });
    expect(v.steps).toHaveLength(18);
    expect(v.steps.every((s) => s.status === 'succeeded')).toBe(true);
    expect(v.steps.find((s) => s.nodeId === 'n_resize')?.count).toBe(9);
    expect(v.clips.map((c) => c.title)).toEqual(['Larkspur Linen Tote', 'Fieldnote Ceramic Mug', 'Meadow Soy Candle']);
    expect(v.clips.every((c) => c.video && c.url.endsWith('.mp4'))).toBe(true);
    expect(v.approval).toEqual({ id: 'apr_fa', nodeId: 'n_approve', decision: 'approve', by: 'Priya (Slack)' });
  });

  it('shows a waiting step and a failure', () => {
    const at = (seq: number, nodeId: string | null, type: string, data: object) =>
      ({ seq, ts: '2026-10-26T09:00:00Z', runId: 'r', nodeId, type, data }) as RunEvent;
    const waiting = replay([at(1, 'n_a', 'node.started', {}), at(2, 'n_a', 'node.waiting', { approvalId: 'apr_1' })]);
    expect(waiting.steps[0]?.status).toBe('waiting');
    expect(waiting.approval).toEqual({ id: 'apr_1', nodeId: 'n_a' });
    const failed = replay([at(1, 'n_a', 'node.failed', { error: 'boom', retryable: false }), at(2, null, 'run.failed', { error: 'n_a failed' })]);
    expect(failed.steps[0]).toMatchObject({ status: 'failed', error: 'boom' });
    expect(failed).toMatchObject({ status: 'failed', error: 'n_a failed' });
  });
});
