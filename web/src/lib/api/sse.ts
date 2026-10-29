// SSE helper. All live updates go through `subscribe` (AGENTS.md §7).
// Live: EventSource with an `after=<seq>` cursor on reconnect. Fixture mode: replays a recorded
// stream with its original timing (sped up by VITE_FIXTURE_SPEED). Unknown event types are passed
// through; consumers must ignore types they don't know (PRD §8.4).

import { isFixtureMode, loadFixture, streamKeys } from './fixtures';

/** Minimal shape every run/build event has. */
export interface StreamEvent {
  seq: number;
  ts: string;
  type: string;
  nodeId?: string | null;
  data?: unknown;
}

export interface Subscription {
  close(): void;
}

export interface SubscribeOptions {
  /** Resume after this sequence number. */
  after?: number;
  /** Called once the stream ends (fixture replay finished, or a terminal event arrived). */
  onEnd?: () => void;
  /** Replay speed multiplier in fixture mode (default from VITE_FIXTURE_SPEED or 1). */
  speed?: number;
  /** Fixture mode: stop the replay this many ms after the first event (to freeze a moment). */
  until?: number;
  /** Called instead of reconnecting when the server refuses the stream (4xx: not found, not yours). */
  onError?: (status: number) => void;
}

const TERMINAL = new Set(['run.succeeded', 'run.failed', 'run.cancelled', 'build.ready', 'build.failed']);

/** Subscribes to `/api/runs/:id/events` or `/api/builds/:id/events`. */
export function subscribe<E extends StreamEvent = StreamEvent>(
  path: string,
  onEvent: (event: E) => void,
  opts: SubscribeOptions = {}
): Subscription {
  return isFixtureMode() ? replayFixture(path, onEvent, opts) : live(path, onEvent, opts);
}

function live<E extends StreamEvent>(path: string, onEvent: (e: E) => void, opts: SubscribeOptions): Subscription {
  let lastSeq = opts.after ?? 0;
  let source: EventSource | null = null;
  let closed = false;
  let retry: ReturnType<typeof setTimeout> | undefined;

  const open = () => {
    const sep = path.includes('?') ? '&' : '?';
    source = new EventSource(`${path}${sep}after=${lastSeq}`);
    source.onmessage = (msg) => {
      let event: E;
      try {
        event = JSON.parse(msg.data) as E;
      } catch {
        return;
      }
      if (event.seq <= lastSeq) return;
      lastSeq = event.seq;
      onEvent(event);
      if (TERMINAL.has(event.type)) {
        sub.close();
        opts.onEnd?.();
      }
    };
    source.onerror = () => {
      source?.close();
      if (!closed) void refusedOrRetry(`${path}${sep}after=${lastSeq}`);
    };
  };

  // EventSource hides the status; ask once, and give up on a 4xx instead of retrying forever.
  const refusedOrRetry = async (url: string) => {
    const abort = new AbortController();
    const status = await fetch(url, { signal: abort.signal }).then((r) => r.status, () => 0);
    abort.abort();
    if (closed) return;
    if (status >= 400 && status < 500) {
      closed = true;
      opts.onError?.(status);
    } else retry = setTimeout(open, 1000);
  };

  const sub: Subscription = {
    close() {
      closed = true;
      clearTimeout(retry);
      source?.close();
    }
  };
  open();
  return sub;
}

/** Delay between two events in a recorded stream, capped so demos never stall. */
export function replayDelay(prevTs: string | undefined, ts: string, speed: number): number {
  if (!prevTs) return 0;
  const gap = Date.parse(ts) - Date.parse(prevTs);
  if (!Number.isFinite(gap) || gap <= 0) return 0;
  return Math.min(gap / speed, 2500);
}

function replayFixture<E extends StreamEvent>(
  path: string,
  onEvent: (e: E) => void,
  opts: SubscribeOptions
): Subscription {
  let closed = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const speed = opts.speed ?? Number(import.meta.env.VITE_FIXTURE_SPEED ?? 1);

  void loadFixture<E[]>(streamKeys(path)).then((events) => {
    if (!events) {
      console.warn(`[fixtures] no recorded stream for ${path}`);
      opts.onEnd?.();
      return;
    }
    const t0 = events.length ? Date.parse(events[0]!.ts) : 0;
    const pending = events.filter(
      (e) => e.seq > (opts.after ?? 0) && (opts.until === undefined || Date.parse(e.ts) - t0 <= opts.until)
    );
    let i = 0;
    let prev: string | undefined;
    const next = () => {
      if (closed) return;
      const event = pending[i++];
      if (!event) {
        opts.onEnd?.();
        return;
      }
      timer = setTimeout(() => {
        if (closed) return;
        prev = event.ts;
        onEvent(event);
        next();
      }, replayDelay(prev, event.ts, speed || 1));
    };
    next();
  });

  return {
    close() {
      closed = true;
      clearTimeout(timer);
    }
  };
}
