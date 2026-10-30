// Getting-started checklist (Q35). Each step ticks itself off from the workspace's flows and connections.

import type { ConnectionSummary, FlowSummary } from '$lib/api';

/** One getting-started step. */
export interface ChecklistStep {
  id: 'connect' | 'build' | 'test' | 'live';
  label: string;
  hint: string;
  /** Where to go to do it. */
  href: string;
  done: boolean;
}

/** The flow the tour and the checklist point at: the first AI app, else the first flow. */
export function focusFlow(flows: FlowSummary[]): FlowSummary | null {
  return flows.find((f) => f.kind === 'app') ?? flows[0] ?? null;
}

/** The four first steps, in order, ticked from what the workspace already has. */
export function checklist(flows: FlowSummary[], connections: ConnectionSummary[]): ChecklistStep[] {
  const flow = focusFlow(flows);
  return [
    {
      id: 'connect',
      label: 'Connect an app',
      hint: 'Sign in to Google, add a bot token or an API key.',
      href: '/connections',
      done: connections.some((c) => c.status === 'connected')
    },
    {
      id: 'build',
      label: 'Build your first flow',
      hint: 'Describe a goal; Interlock builds and tests it.',
      href: '/new',
      done: flows.length > 0
    },
    {
      id: 'test',
      label: 'Run a test',
      hint: 'Try it with sample data. Nothing is sent for real.',
      href: !flow ? '/new' : flow.kind === 'app' ? `/flows/${flow.id}/chat` : `/flows/${flow.id}`,
      done: flows.some((f) => f.lastRun !== null)
    },
    {
      id: 'live',
      label: 'Turn it on or publish',
      hint: 'Switch an automation on, or publish an AI app.',
      href: flow ? `/flows/${flow.id}/publish` : '/new',
      done: flows.some((f) => f.active)
    }
  ];
}
