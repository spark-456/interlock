// One playground turn: the question, then everything its test run shows (route, tools, searches,
// the path through the flow) and the answer with its sources and suggested actions.

import type { Flow, RunEvent } from '$lib/api/types';

export interface Passage {
  text: string;
  score: number;
  sourceId: string;
  title: string;
  section?: string | null;
  url?: string | null;
}

export interface Source {
  sourceId: string;
  title: string;
  section?: string | null;
  url?: string | null;
}

export interface Turn {
  question: string;
  status: 'running' | 'done' | 'failed';
  /** "Routed to Billing", from a router step's output port. */
  route?: string;
  /** Steps that ran, in order: the path through the flow. */
  path: { nodeId: string; name: string; ms?: number }[];
  tools: string[];
  passages: Passage[];
  answer?: string;
  /** The step that wrote the answer, and how long it took. */
  by?: { name: string; ms?: number };
  sources: Source[];
  actions: { label: string; message: string }[];
  /** Set when a safety step stopped the answer. */
  guardrail?: string;
  error?: string;
}

export const newTurn = (question: string): Turn => ({ question, status: 'running', path: [], tools: [], passages: [], sources: [], actions: [] });

const ROUTERS = new Set(['ai.decide', 'ai.classify', 'logic.switch']);
const BLOCKED = new Set(['blocked', 'no', 'false']);

/** Folds one run event into the turn. `flow` names steps and tells routers and guards apart. */
export function reduceTurn(t: Turn, e: RunEvent, flow: Flow): Turn {
  const node = flow.nodes.find((n) => n.id === e.nodeId);
  const name = node?.name ?? e.nodeId ?? '';
  switch (e.type) {
    case 'node.started':
      return t.path.some((p) => p.nodeId === e.nodeId) ? t : { ...t, path: [...t.path, { nodeId: e.nodeId ?? '', name }] };
    case 'node.succeeded':
      return {
        ...t,
        path: t.path.map((p) => (p.nodeId === e.nodeId ? { ...p, ms: e.data.ms } : p)),
        by: t.by && t.by.name === name ? { ...t.by, ms: e.data.ms } : t.by
      };
    case 'node.items': {
      const json = (e.data.items[0]?.json ?? {}) as Record<string, unknown>;
      // A safety step, or a router that runs after the answer, checks it rather than routing.
      if (node && (node.type === 'ai.guardrails' || (ROUTERS.has(node.type) && t.answer !== undefined))) {
        return BLOCKED.has(e.data.port) ? { ...t, guardrail: name, answer: str(json.answer) ?? t.answer } : t;
      }
      if (node && ROUTERS.has(node.type)) return { ...t, route: e.data.port };
      if (typeof json.answer !== 'string') return t;
      const activity = Array.isArray(json.activity) ? (json.activity as { tool?: string; result?: unknown }[]) : [];
      const searches = activity.filter((a) => a.tool?.startsWith('search_'));
      return {
        ...t,
        answer: json.answer,
        by: { name },
        sources: Array.isArray(json.sources) ? (json.sources as Source[]) : t.sources,
        actions: Array.isArray(json.actions) ? (json.actions as Turn['actions']) : t.actions,
        tools: [...new Set([...t.tools, ...activity.filter((a) => a.tool && !a.tool.startsWith('search_')).map((a) => a.tool!)])],
        passages: [...t.passages, ...searches.flatMap((s) => (Array.isArray(s.result) ? (s.result as Passage[]) : []))]
      };
    }
    case 'node.failed':
      return { ...t, error: `${name}: ${e.data.error}` };
    case 'run.succeeded':
      return { ...t, status: 'done' };
    case 'run.failed':
      return { ...t, status: 'failed', error: t.error ?? e.data.error };
    default:
      return t;
  }
}

const str = (v: unknown) => (typeof v === 'string' ? v : undefined);

/** The passage a source chip opens: the best-scoring hit from that source. */
export function passageFor(t: Turn, sourceId: string): { passage: Passage; rank: number; of: number } | undefined {
  const ranked = [...t.passages].sort((a, b) => b.score - a.score);
  const i = ranked.findIndex((p) => p.sourceId === sourceId);
  return i < 0 ? undefined : { passage: ranked[i]!, rank: i + 1, of: ranked.length };
}

/** "Try asking" prompts: the chat trigger's suggested questions, or a generic opener. */
export function suggestions(flow: Flow): string[] {
  const chat = flow.nodes.find((n) => n.type === 'trigger.chat')?.settings as { suggestedQuestions?: unknown } | undefined;
  const list = Array.isArray(chat?.suggestedQuestions) ? chat.suggestedQuestions.filter((q): q is string => typeof q === 'string') : [];
  return list.length ? list : ['What can you help me with?'];
}
