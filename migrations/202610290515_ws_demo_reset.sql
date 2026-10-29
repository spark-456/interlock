CREATE TABLE ws_demo_workspaces (
  workspace_id TEXT PRIMARY KEY, created_at TEXT NOT NULL
);
CREATE INDEX ws_demo_workspaces_created_at ON ws_demo_workspaces(created_at);
INSERT OR IGNORE INTO ws_demo_workspaces (workspace_id, created_at)
SELECT DISTINCT json_extract(value, '$.workspaceId'), strftime('%Y-%m-%dT%H:%M:%S+00:00', 'now')
FROM tr_poll_cursors
WHERE key LIKE 'ws:session:%' AND json_extract(value, '$.demo') = 1;
ALTER TABLE fs_files ADD COLUMN workspace_id TEXT;
CREATE INDEX fs_files_workspace ON fs_files(workspace_id);
