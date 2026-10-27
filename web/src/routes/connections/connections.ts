import type { ConnectionSummary, CredentialKind, NodeManifest } from '$lib/api/types';

/** One app on the connections grid. */
export interface AppRow {
  app: string;
  /** "Google", from the connection or the first word of the credential label. */
  name: string;
  /** What gets connected: "Google account", "Telegram bot token". */
  label: string;
  kind: CredentialKind;
  status: ConnectionSummary['status'];
  account: string | null;
}

/** Generic HTTP and OpenAPI keys belong to a step, not to an app, so they get no tile. */
const PER_STEP = new Set(['http', 'openapi']);

/** Every app the catalog can connect to, with its connection state; connected apps first. */
export function appRows(nodes: NodeManifest[], connections: ConnectionSummary[]): AppRow[] {
  const rows = new Map<string, AppRow>();
  for (const spec of nodes.flatMap((n) => n.credentials)) {
    if (PER_STEP.has(spec.app) || rows.has(spec.app)) continue;
    rows.set(spec.app, { app: spec.app, name: spec.label.split(' ')[0] ?? spec.app, label: spec.label, kind: spec.kind, status: 'notConnected', account: null });
  }
  for (const c of connections) {
    const row = rows.get(c.app);
    if (row) rows.set(c.app, { ...row, name: c.label, status: c.status, account: c.account });
  }
  const rank = { connected: 0, expired: 1, notConnected: 2 };
  return [...rows.values()].sort((a, b) => rank[a.status] - rank[b.status] || a.name.localeCompare(b.name));
}

/** Where the Connect button goes: the OAuth start route, or nowhere for key-based apps. */
export const connectHref = (row: AppRow) => (row.kind === 'oAuth2' ? `/api/oauth/${row.app}/start` : null);
