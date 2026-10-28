// Typed endpoints (PRD §8.5). Add endpoints here as screens need them; types are generated.

import { del, get, post, put, upload } from './client';
import type {
  Approval,
  ApprovalDecision,
  AskDiff,
  BuildAnswers,
  BuildCreated,
  BuildEvent,
  BuildRequest,
  CatalogResponse,
  ConnectionSummary,
  CreateConnectionRequest,
  FileRef,
  Flow,
  FlowSummary,
  JsonValue,
  Me,
  RunCreated,
  RunEvent,
  RunSummary,
  TemplateSummary,
  UsageReport
} from './types';
import { subscribe, type SubscribeOptions } from './sse';

export { ApiRequestError } from './client';
export { isFixtureMode } from './fixtures';
export type * from './types';

export const api = {
  nodes: () => get<CatalogResponse>('/api/nodes'),
  me: () => get<Me>('/api/me'),
  flows: () => get<FlowSummary[]>('/api/flows'),
  flow: (id: string) => get<Flow>(`/api/flows/${id}`),
  saveFlow: (flow: Flow) => put<Flow>(`/api/flows/${flow.id}`, flow),
  ask: (flowId: string, message: string) => post<AskDiff>(`/api/flows/${flowId}/ask`, { message }),
  activate: (flowId: string) => post<void>(`/api/flows/${flowId}/activate`, {}),
  deactivate: (flowId: string) => post<void>(`/api/flows/${flowId}/deactivate`, {}),
  /** The integration guide (INTERLOCK.md) for a published AI app. */
  guide: (flowId: string) => get<{ markdown: string }>(`/api/flows/${flowId}/guide`),
  /** Starts an anonymous demo workspace (sets the session cookie). */
  demoSession: () => post<Me>('/api/demo/session', {}),
  keys: () => get<{ id: string; name: string; revoked: boolean; lastUsedAt: string | null }[]>('/api/keys'),
  revokeKey: (id: string) => del<void>(`/api/keys/${id}`),
  /** Creates an API key; the full key is in this response only. */
  createKey: (name: string) => post<{ id: string; key: string }>('/api/keys', { name }),
  testRun: (flowId: string, items: JsonValue[] = []) => post<RunCreated>(`/api/flows/${flowId}/test`, { items }),
  runStep: (flowId: string, nodeId: string, items: JsonValue[]) =>
    post<RunCreated>(`/api/flows/${flowId}/nodes/${nodeId}/run`, { items }),
  connections: () => get<ConnectionSummary[]>('/api/connections'),
  usage: () => get<UsageReport>('/api/usage'),
  /** Saves a key-based connection; the secret is encrypted on the server and never sent back. */
  createConnection: (req: CreateConnectionRequest) => post<ConnectionSummary>('/api/connections', req),
  runs: (flowId: string) => get<RunSummary[]>(`/api/flows/${flowId}/runs`),
  approvals: () => get<Approval[]>('/api/approvals'),
  decide: (approvalId: string, decision: ApprovalDecision) => post<void>(`/api/approvals/${approvalId}/decide`, decision),
  templates: () => get<TemplateSummary[]>('/api/templates'),
  /** Copies a template into the workspace; returns the new flow. */
  useTemplate: (id: string) => post<Flow>(`/api/templates/${id}/use`, {}),
  startBuild: (req: BuildRequest) => post<BuildCreated>('/api/builds', req),
  answerBuild: (buildId: string, answers: BuildAnswers) => post<void>(`/api/builds/${buildId}/answers`, answers),
  uploadFiles: (files: File[]) => upload<FileRef[]>('/api/files', files)
};

/** Live build events for the Building screen. */
export const buildEvents = (id: string, onEvent: (e: BuildEvent) => void, opts?: SubscribeOptions) =>
  subscribe<BuildEvent>(`/api/builds/${id}/events`, onEvent, opts);

/** Live run events for the editor and run view. */
export const runEvents = (id: string, onEvent: (e: RunEvent) => void, opts?: SubscribeOptions) =>
  subscribe<RunEvent>(`/api/runs/${id}/events`, onEvent, opts);

/** App display names from the catalog: "Reply (Gmail)" → gmail: Gmail. */
export async function appNames(): Promise<Record<string, string>> {
  const c = await api.nodes();
  return Object.fromEntries(c.nodes.filter((n) => n.app).map((n) => [n.app!, n.title.match(/\(([^)]+)\)$/)?.[1] ?? n.app!]));
}
