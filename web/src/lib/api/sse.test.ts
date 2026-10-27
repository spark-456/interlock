import { describe, expect, it } from 'vitest';
import { replayDelay } from './sse';
import { fixtureKeys, streamKeys } from './fixtures';

describe('fixture paths', () => {
  it('maps GET and POST paths to files with a default fallback', () => {
    expect(fixtureKeys('GET', '/api/flows/flw_promos?x=1')).toEqual([
      '/fixtures/api/flows/flw_promos.json',
      '/fixtures/api/flows/_default.json'
    ]);
    expect(fixtureKeys('POST', '/api/builds')).toEqual(['/fixtures/api/builds.post.json', '/fixtures/api/_default.post.json']);
  });

  it('maps SSE paths to recorded streams', () => {
    expect(streamKeys('/api/builds/bld_fa/events?after=3')[0]).toBe('/fixtures/streams/builds/bld_fa.json');
    expect(streamKeys('/api/runs/run_fa/events')[0]).toBe('/fixtures/streams/runs/run_fa.json');
    expect(streamKeys('/api/flows')).toEqual([]);
  });
});

describe('replayDelay', () => {
  it('uses the recorded gap, scaled and capped', () => {
    expect(replayDelay(undefined, '2026-10-27T09:00:00Z', 1)).toBe(0);
    expect(replayDelay('2026-10-27T09:00:00Z', '2026-10-27T09:00:01Z', 1)).toBe(1000);
    expect(replayDelay('2026-10-27T09:00:00Z', '2026-10-27T09:00:01Z', 4)).toBe(250);
    expect(replayDelay('2026-10-27T09:00:00Z', '2026-10-27T09:01:00Z', 1)).toBe(2500);
  });
});
