// Guided tour (Q35): coach-marks over the real screens. Steps that need a flow use the workspace's
// first AI app (else its first flow); without one they show as a centred card wherever the tour is.

import type { FlowSummary } from '$lib/api';

/** One coach-mark. */
export interface TourStep {
  title: string;
  body: string;
  /** The screen it is shown on; `:flow` is the tour's flow. */
  page: string;
  /** Elements to point at, first visible one wins; with none the card is centred. */
  target: string[];
  /** Only an AI app has this screen. */
  app?: boolean;
}

export const TOUR: TourStep[] = [
  {
    title: 'Say what it should do',
    body: 'Describe the goal in a sentence or two: what starts it, what it reads, what it makes and where the result goes. Type it, pick an example, or use the mic.',
    page: '/new',
    target: ['section[aria-labelledby="s1"]']
  },
  {
    title: 'Give it what it should know',
    body: 'Drop files, links or voice memos, then mark each one. Knowledge is searched when a question needs it, like help articles or a price list. Instructions are always followed, like tone, rules or a process.',
    page: '/new',
    target: ['section[aria-labelledby="s2"]']
  },
  {
    title: 'Apps and APIs',
    body: 'Apps you mention in the goal show up here; click one to leave it out. Drop an OpenAPI spec and its safe actions become tools.',
    page: '/new',
    target: ['section[aria-labelledby="s3"]']
  },
  {
    title: 'Build it and watch',
    body: 'Build it opens the Build view: Interlock plans the steps, wires them up and tests them while you watch. If it needs something from you, it asks. When it is done, the flow opens in the editor.',
    page: '/new',
    target: ['[data-tour="build"]']
  },
  {
    title: 'Two kinds of flow',
    body: 'Automations run on their own when something happens, like a schedule, an email or a webhook. AI apps answer people. Each card shows whether it is on and its last run; open a run to see every step’s input and output.',
    page: '/flows',
    target: ['[data-tour="flows"]']
  },
  {
    title: 'The canvas',
    body: 'Each box is a step, and the lines show how items move between them. Drag steps to rearrange them; undo, tidy and zoom sit in the corner.',
    page: '/flows/:flow',
    target: ['[data-tour="canvas"]']
  },
  {
    title: 'The step panel',
    body: 'Click a step to open its panel: its settings, what goes in and what comes out. “Run this step” tries just that one.',
    page: '/flows/:flow',
    target: ['.svelte-flow__node', '[data-tour="canvas"]']
  },
  {
    title: 'Add step',
    body: 'Pick a step from the library, or drag one onto a line to put it between two steps.',
    page: '/flows/:flow',
    target: ['[data-tour="add-step"]']
  },
  {
    title: 'Refine in plain words',
    body: 'Ask for a change, like “also post it to Slack”. Type @ to pick the steps you mean, and only those change. The proposal shows on the canvas: accept or discard it, then keep refining.',
    page: '/flows/:flow',
    target: ['[data-tour="refine"]']
  },
  {
    title: 'Test run',
    body: 'Runs the flow with sample data and lights up each step as it goes. Test runs never post or send for real, so try it as often as you like.',
    page: '/flows/:flow',
    target: ['[data-tour="test-run"]']
  },
  {
    title: 'The Playground',
    body: 'Chat with an AI app. Each message is a test run: you see the route it took, the tools it called and the sources behind the answer.',
    page: '/flows/:flow/chat',
    target: ['section[aria-label="Chat"] form'],
    app: true
  },
  {
    title: 'Approvals',
    body: 'Steps that send or post can wait for your OK. They land here, on desktop or phone: approve, edit or reject, and the run carries on.',
    page: '/approvals',
    target: ['section[aria-label="Waiting"] > *', 'main h1']
  },
  {
    title: 'Connections',
    body: 'Sign in to Google once for Gmail, Sheets and YouTube. Paste a bot token for Slack, Telegram or Discord, or an API key for other services. Secrets are encrypted and never shown again.',
    page: '/connections',
    target: ['ul[aria-label="Apps"]']
  },
  {
    title: 'Publish',
    body: 'Turn an automation on, or publish an AI app: call it over the OpenAI-compatible API, add it to Claude Code or Codex as an MCP server, or paste the chat widget into your site.',
    page: '/flows/:flow/publish',
    target: ['[data-tour="publish"]', 'main section']
  },
  {
    title: 'That’s the tour',
    body: 'Help, in the sidebar, takes you through it again and shows your getting-started checklist. On a phone, both are in Settings.',
    page: '/flows',
    target: ['[data-tour="help"]']
  }
];

/** The path a step is shown on, or null when the workspace has no flow (or no AI app) for it. */
export function stepPath(step: TourStep, flow: FlowSummary | null): string | null {
  if (!step.page.includes(':flow')) return step.page;
  if (!flow || (step.app && flow.kind !== 'app')) return null;
  return step.page.replace(':flow', flow.id);
}

/** A rectangle in viewport pixels. */
export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/**
 * Top-left corner of the coach-mark card: under the target when it fits, else above it, else docked
 * to the bottom of the screen (a target that fills the screen, like the canvas); centred with no target.
 */
export function place(target: Box | null, card: { w: number; h: number }, view: { w: number; h: number }, gap = 12, margin = 16): { x: number; y: number } {
  const clampX = (x: number) => Math.max(margin, Math.min(x, view.w - card.w - margin));
  if (!target) return { x: clampX((view.w - card.w) / 2), y: Math.max(margin, (view.h - card.h) / 2) };
  const x = clampX(target.x + target.w / 2 - card.w / 2);
  const below = target.y + target.h + gap;
  if (below + card.h <= view.h - margin) return { x, y: below };
  const above = target.y - gap - card.h;
  if (above >= margin) return { x, y: above };
  return { x, y: Math.max(margin, view.h - card.h - margin) };
}
