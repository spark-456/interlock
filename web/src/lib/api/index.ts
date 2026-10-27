// Typed endpoints (PRD §8.5). Add endpoints here as screens need them; types are generated.

import { get, post } from './client';
import type {
  BuildCreated,
  BuildEvent,
  BuildRequest,
  CatalogResponse,
  Flow,
  FlowSummary,
  Me,
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
  runs: (flowId: string) => get<RunSummary[]>(`/api/flows/${flowId}/runs`),
  templates: () => get<TemplateSummary[]>('/api/templates'),
  startBuild: (req: BuildRequest) => post<BuildCreated>('/api/builds', req)
};

/** Live build events for the Building screen. */
export const buildEvents = (id: string, onEvent: (e: BuildEvent) => void, opts?: SubscribeOptions) =>
  subscribe<BuildEvent>(`/api/builds/${id}/events`, onEvent, opts);

/** Live run events for the editor and run view. */
export const runEvents = (id: string, onEvent: (e: RunEvent) => void, opts?: SubscribeOptions) =>
  subscribe<RunEvent>(`/api/runs/${id}/events`, onEvent, opts);
