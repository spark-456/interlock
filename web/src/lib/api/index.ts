// Typed endpoints (PRD §8.5). Add endpoints here as screens need them; types are generated.

import { get, post, put, upload } from './client';
import type {
  AskDiff,
  BuildAnswers,
  BuildCreated,
  BuildEvent,
  BuildRequest,
  CatalogResponse,
  ConnectionSummary,
  FileRef,
  Flow,
  FlowSummary,
  JsonValue,
  Me,
  RunCreated,
  RunEvent,
  RunSummary,
  TemplateSummary
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
  testRun: (flowId: string) => post<RunCreated>(`/api/flows/${flowId}/test`, { items: [] }),
  runStep: (flowId: string, nodeId: string, items: JsonValue[]) =>
    post<RunCreated>(`/api/flows/${flowId}/nodes/${nodeId}/run`, { items }),
  connections: () => get<ConnectionSummary[]>('/api/connections'),
  runs: (flowId: string) => get<RunSummary[]>(`/api/flows/${flowId}/runs`),
  templates: () => get<TemplateSummary[]>('/api/templates'),
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
