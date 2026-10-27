import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

// Every recorded stream must be ordered and use known PRD §8.4 event types.
const RUN = new Set(['run.started', 'node.started', 'node.progress', 'node.items', 'node.succeeded', 'node.failed',
  'node.waiting', 'approval.decided', 'run.cap_reached', 'run.succeeded', 'run.failed', 'run.cancelled']);
const BUILD = new Set(['build.stage', 'build.log', 'build.question', 'build.flow_patch', 'build.connection_needed',
  'build.test_result', 'build.fix', 'build.ready', 'build.failed', 'build.node', 'build.cost']);

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? files(p) : [p];
  });
}

describe('recorded streams', () => {
  const root = join(process.cwd(), 'fixtures/streams');
  for (const file of files(root)) {
    it(`${file.replace(root, '')} is ordered and typed`, () => {
      const events = JSON.parse(readFileSync(file, 'utf8')) as { seq: number; ts: string; type: string }[];
      expect(events.length).toBeGreaterThan(3);
      const known = file.includes('builds') ? BUILD : RUN;
      let prev = 0;
      for (const e of events) {
        expect(e.seq).toBeGreaterThan(prev);
        expect(known.has(e.type), e.type).toBe(true);
        expect(Number.isFinite(Date.parse(e.ts))).toBe(true);
        prev = e.seq;
      }
    });
  }
});
