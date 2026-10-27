CREATE TABLE IF NOT EXISTS ws_workspaces (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS ws_users (
  id TEXT PRIMARY KEY, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS ws_members (
  workspace_id TEXT NOT NULL, user_id TEXT NOT NULL, role TEXT NOT NULL,
  PRIMARY KEY (workspace_id, user_id),
  FOREIGN KEY (workspace_id) REFERENCES ws_workspaces(id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES ws_users(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS ws_api_keys (
  id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, name TEXT NOT NULL,
  key_hash BLOB NOT NULL UNIQUE, created_at TEXT NOT NULL,
  last_used_at TEXT, revoked_at TEXT
);
CREATE INDEX IF NOT EXISTS ws_api_keys_workspace ON ws_api_keys(workspace_id);

CREATE TABLE IF NOT EXISTS fl_flows (
  id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, version INTEGER NOT NULL,
  active INTEGER NOT NULL DEFAULT 0, body TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS fl_flows_workspace ON fl_flows(workspace_id);
CREATE TABLE IF NOT EXISTS fl_versions (
  flow_id TEXT NOT NULL, version INTEGER NOT NULL, body TEXT NOT NULL,
  PRIMARY KEY (flow_id, version)
);

CREATE TABLE IF NOT EXISTS rn_runs (
  id TEXT PRIMARY KEY, flow_id TEXT NOT NULL, workspace_id TEXT NOT NULL,
  status TEXT NOT NULL, started_at TEXT NOT NULL, body TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS rn_runs_flow_time ON rn_runs(flow_id, started_at DESC);
CREATE INDEX IF NOT EXISTS rn_runs_recovery ON rn_runs(status);
CREATE TABLE IF NOT EXISTS rn_events (
  run_id TEXT NOT NULL, seq INTEGER NOT NULL, body TEXT NOT NULL,
  PRIMARY KEY (run_id, seq)
);
CREATE TABLE IF NOT EXISTS rn_node_runs (
  run_id TEXT NOT NULL, node_id TEXT NOT NULL, body TEXT NOT NULL,
  PRIMARY KEY (run_id, node_id)
);
CREATE TABLE IF NOT EXISTS rn_approvals (
  id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, status TEXT NOT NULL,
  created_at TEXT NOT NULL, body TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS rn_approvals_inbox ON rn_approvals(workspace_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS tr_schedules (
  id TEXT PRIMARY KEY, flow_id TEXT NOT NULL, cron TEXT NOT NULL,
  timezone TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS tr_webhooks (
  id TEXT PRIMARY KEY, flow_id TEXT NOT NULL, token_hash BLOB NOT NULL UNIQUE,
  enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS tr_poll_cursors (
  key TEXT PRIMARY KEY, value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cn_credentials (
  id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, app TEXT NOT NULL,
  nonce BLOB NOT NULL, ciphertext BLOB NOT NULL, meta TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS cn_credentials_workspace ON cn_credentials(workspace_id);
CREATE TABLE IF NOT EXISTS cn_oauth_states (
  state_hash BLOB PRIMARY KEY, workspace_id TEXT NOT NULL, app TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fs_files (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, mime TEXT NOT NULL,
  bytes INTEGER NOT NULL, path TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS bd_builds (
  id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, status TEXT NOT NULL,
  created_at TEXT NOT NULL, body TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS bd_events (
  build_id TEXT NOT NULL, seq INTEGER NOT NULL, body TEXT NOT NULL,
  PRIMARY KEY (build_id, seq)
);
CREATE TABLE IF NOT EXISTS us_ledger (
  id INTEGER PRIMARY KEY, workspace_id TEXT NOT NULL, flow_id TEXT,
  run_id TEXT, build_id TEXT, role TEXT NOT NULL,
  cost_usd REAL NOT NULL CHECK(cost_usd >= 0), at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS us_run ON us_ledger(run_id);
CREATE INDEX IF NOT EXISTS us_flow_time ON us_ledger(flow_id, at);
CREATE INDEX IF NOT EXISTS us_workspace_time ON us_ledger(workspace_id, at);
CREATE TABLE IF NOT EXISTS pb_endpoints (
  id TEXT PRIMARY KEY, flow_id TEXT NOT NULL, slug TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);
