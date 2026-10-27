// Build session state, reduced from build SSE events (PRD §8.4 + the additive `build.node` and
// `build.cost`). Pure: the Building screen and tests call `reduce` for every event.

import type { BuildEvent, BuildNodeStatus, BuildQuestion, Edge, Group, Node, NodeStat, StageStatus } from '$lib/api/types';
import { applyPatch, type PatchOp } from './jsonPatch';

/** The seven steps of the stepper, in order (key, desktop label, mobile label). */
export const STAGES = [
  { key: 'read', label: 'Read', short: 'Read' },
  { key: 'index', label: 'Index', short: 'Index' },
  { key: 'team', label: 'Design team', short: 'Team' },
  { key: 'prompts', label: 'Write prompts', short: 'Write' },
  { key: 'tools', label: 'Wire tools', short: 'Wire' },
  { key: 'test', label: 'Test', short: 'Test' },
  { key: 'live', label: 'Go live', short: 'Live' }
] as const;

/** Stages of an automation build (the architect's `understand` → `assemble` → `test`). */
const AUTOMATION_STAGES = [
  { key: 'understand', label: 'Understand the goal', short: 'Read' },
  { key: 'assemble', label: 'Assemble the steps', short: 'Build' },
  { key: 'test', label: 'Test', short: 'Test' }
] as const;

export type Stage = { key: string; label: string; short: string };

/** The stage list this build reports: the design's AI-app stages, or the automation ones. */
export function stagesOf(s: BuildState): readonly Stage[] {
  return AUTOMATION_STAGES.some((st) => st.key !== 'test' && s.stages[st.key]) ? AUTOMATION_STAGES : STAGES;
}

export interface NodeState {
  status: BuildNodeStatus;
  subtitle?: string;
  label?: string;
  summary?: string;
  stats: NodeStat[];
  progress?: number;
  icon?: string;
  tone?: string;
}

export interface Fix {
  description: string;
  title?: string;
  before?: string;
  after?: string;
  source?: string;
  nodeId?: string | null;
}

export interface BuildState {
  flow: { name?: string; nodes: Node[]; edges: Edge[]; groups: Group[] };
  nodes: Record<string, NodeState>;
  stages: Record<string, { status: StageStatus; start?: number; end?: number }>;
  log: { t: number; text: string }[];
  tests: { passed: number; failed: number } | null;
  fixes: Fix[];
  connections: string[];
  /** Open clarifying questions; cleared once the build moves on. */
  questions: BuildQuestion[];
  costUsd: number;
  /** Epoch ms of the first event, and elapsed ms of the latest one. */
  t0: number | null;
  elapsed: number;
  status: 'building' | 'ready' | 'failed';
  flowId: string | null;
  error: string | null;
  lastSeq: number;
}

export function initialBuildState(): BuildState {
  return {
    flow: { nodes: [], edges: [], groups: [] },
    nodes: {},
    stages: {},
    log: [],
    tests: null,
    fixes: [],
    connections: [],
    questions: [],
    costUsd: 0,
    t0: null,
    elapsed: 0,
    status: 'building',
    flowId: null,
    error: null,
    lastSeq: 0
  };
}

/** Applies one event. Unknown types are ignored (forward compatible). Returns a new state. */
export function reduce(prev: BuildState, ev: BuildEvent): BuildState {
  if (ev.seq <= prev.lastSeq) return prev;
  const s: BuildState = { ...prev, lastSeq: ev.seq };
  // Questions stay open while only the log and cost change; anything else means the build moved on.
  if (ev.type !== 'build.log' && ev.type !== 'build.cost') s.questions = [];
  const at = Date.parse(ev.ts);
  s.t0 = prev.t0 ?? at;
  s.elapsed = Math.max(prev.elapsed, at - s.t0);
  const t = s.elapsed;

  switch (ev.type) {
    case 'build.flow_patch': {
      const ops = (Array.isArray(ev.data.jsonPatch) ? ev.data.jsonPatch : []) as unknown as PatchOp[];
      s.flow = applyPatch(prev.flow, ops);
      for (const n of s.flow.nodes) {
        if (!s.nodes[n.id]) s.nodes = { ...s.nodes, [n.id]: { status: 'waiting', stats: [] } };
      }
      break;
    }
    case 'build.node': {
      if (!ev.nodeId) break;
      const old = prev.nodes[ev.nodeId];
      const d = ev.data;
      s.nodes = {
        ...prev.nodes,
        [ev.nodeId]: {
          status: d.status,
          subtitle: d.subtitle ?? old?.subtitle,
          label: d.label,
          summary: d.summary ?? old?.summary,
          stats: d.stats ?? old?.stats ?? [],
          progress: d.progress,
          icon: d.icon ?? old?.icon,
          tone: d.tone ?? old?.tone
        }
      };
      break;
    }
    case 'build.stage': {
      const old = prev.stages[ev.data.stage];
      s.stages = {
        ...prev.stages,
        [ev.data.stage]: {
          status: ev.data.status,
          start: old?.start ?? (ev.data.status === 'active' ? t : undefined),
          end: ev.data.status === 'done' || ev.data.status === 'failed' ? t : old?.end
        }
      };
      break;
    }
    case 'build.log':
      s.log = [...prev.log, { t, text: ev.data.text }];
      break;
    case 'build.test_result':
      s.tests = { passed: ev.data.passed, failed: ev.data.failed };
      break;
    case 'build.fix':
      s.fixes = [...prev.fixes, { ...ev.data, nodeId: ev.nodeId }];
      break;
    case 'build.connection_needed':
      if (!prev.connections.includes(ev.data.app)) s.connections = [...prev.connections, ev.data.app];
      break;
    case 'build.question':
      s.questions = ev.data.questions;
      break;
    case 'build.cost':
      s.costUsd = ev.data.costUsd;
      break;
    case 'build.ready':
      s.status = 'ready';
      s.flowId = ev.data.flowId;
      break;
    case 'build.failed':
      s.status = 'failed';
      s.error = ev.data.error;
      break;
    default:
      break;
  }
  return s;
}

/** Index of the active stage (or the next pending one), 0-based. */
export function currentStageIndex(s: BuildState): number {
  const stages = stagesOf(s);
  const active = stages.findIndex((st) => s.stages[st.key]?.status === 'active');
  if (active >= 0) return active;
  const firstOpen = stages.findIndex((st) => s.stages[st.key]?.status !== 'done');
  return firstOpen < 0 ? stages.length - 1 : firstOpen;
}

/** Tests run / total from the test-suite node's "Run" stat (e.g. "21 / 30"). */
export function testsRun(s: BuildState): { run: number; total: number } | null {
  for (const n of Object.values(s.nodes)) {
    const stat = n.stats.find((x) => x.label === 'Run' || x.label === 'Passed');
    const m = stat?.value.match(/^(\d+)\s*\/\s*(\d+)$/);
    if (m) return { run: Number(m[1]), total: Number(m[2]) };
  }
  return null;
}

/** Overall progress 0–1: finished stages plus the active stage's fraction. */
export function progress(s: BuildState): number {
  if (s.status === 'ready') return 1;
  const stages = stagesOf(s);
  const done = stages.filter((st) => s.stages[st.key]?.status === 'done').length;
  const idx = currentStageIndex(s);
  const active = s.stages[stages[idx]!.key]?.status === 'active';
  let frac = active ? 0.5 : 0;
  if (active && stages[idx]!.key === 'test') {
    const r = testsRun(s);
    if (r && r.total > 0) frac = r.run / r.total;
  }
  return Math.min(1, (done + frac) / stages.length);
}

/** Rough time left in ms, rounded to 5 s (from the elapsed time and progress). */
export function timeLeft(s: BuildState): number | null {
  const p = progress(s);
  if (p <= 0.05 || p >= 1) return null;
  const left = (s.elapsed * (1 - p)) / p;
  return Math.max(5000, Math.ceil(left / 5000) * 5000);
}
