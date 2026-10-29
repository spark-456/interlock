//! Demo workspace registration and scoped record cleanup.

use chrono::{DateTime, Utc};
use il_sdk::services::store::StoreResult;
use rusqlite::{OptionalExtension, params};

use crate::db::{SqliteStore, backend};

impl SqliteStore {
    /// Registers one demo workspace at creation.
    pub async fn register_demo_workspace(&self, workspace_id: &str, created_at: DateTime<Utc>) -> StoreResult<()> {
        let (id, at) = (workspace_id.to_owned(), created_at.to_rfc3339());
        self.with_conn(move |c| {
            c.execute("INSERT INTO ws_demo_workspaces (workspace_id,created_at) VALUES (?1,?2)", params![id, at])
                .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Lists demos older than a cutoff; the boundary itself remains valid.
    pub async fn expired_demo_workspaces(&self, cutoff: DateTime<Utc>) -> StoreResult<Vec<String>> {
        let cutoff = cutoff.to_rfc3339();
        self.with_conn(move |c| {
            let mut stmt = c.prepare("SELECT workspace_id FROM ws_demo_workspaces WHERE created_at < ?1 ORDER BY created_at, workspace_id")
                .map_err(backend)?;
            let rows = stmt.query_map([cutoff], |row| row.get(0)).map_err(backend)?;
            rows.collect::<Result<Vec<String>, _>>().map_err(backend)
        }).await
    }

    /// Removes demo data in one transaction after its files have been purged.
    pub async fn delete_demo_workspace(&self, workspace_id: &str) -> StoreResult<()> {
        let id = workspace_id.to_owned();
        self.with_conn(move |c| {
            let tx = c.transaction().map_err(backend)?;
            let registered: Option<i64> = tx.query_row(
                "SELECT 1 FROM ws_demo_workspaces WHERE workspace_id=?1", [&id], |row| row.get(0)
            ).optional().map_err(backend)?;
            if registered.is_none() { return Ok(()); }

            let has_vec: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='kb_chunks_vec')", [], |row| row.get(0)
            ).map_err(backend)?;
            if has_vec {
                tx.execute("DELETE FROM kb_chunks_vec WHERE rowid IN (SELECT c.id FROM kb_chunks c JOIN kb_bases b ON b.id=c.kb_id WHERE b.workspace_id=?1)", [&id]).map_err(backend)?;
            }
            for sql in [
                "DELETE FROM kb_bases WHERE workspace_id=?1",
                "DELETE FROM rn_events WHERE run_id IN (SELECT id FROM rn_runs WHERE workspace_id=?1)",
                "DELETE FROM rn_node_runs WHERE run_id IN (SELECT id FROM rn_runs WHERE workspace_id=?1)",
                "DELETE FROM rn_runs WHERE workspace_id=?1",
                "DELETE FROM rn_approvals WHERE workspace_id=?1",
                "DELETE FROM bd_events WHERE build_id IN (SELECT id FROM bd_builds WHERE workspace_id=?1)",
                "DELETE FROM bd_builds WHERE workspace_id=?1",
                "DELETE FROM tr_schedules WHERE flow_id IN (SELECT id FROM fl_flows WHERE workspace_id=?1)",
                "DELETE FROM tr_webhooks WHERE flow_id IN (SELECT id FROM fl_flows WHERE workspace_id=?1)",
                "DELETE FROM pb_endpoints WHERE flow_id IN (SELECT id FROM fl_flows WHERE workspace_id=?1)",
                "DELETE FROM fl_versions WHERE flow_id IN (SELECT id FROM fl_flows WHERE workspace_id=?1)",
                "DELETE FROM fl_flows WHERE workspace_id=?1",
                "DELETE FROM cn_credentials WHERE workspace_id=?1",
                "DELETE FROM cn_oauth_states WHERE workspace_id=?1",
                "DELETE FROM ws_api_keys WHERE workspace_id=?1",
                "DELETE FROM ws_members WHERE workspace_id=?1",
                "DELETE FROM ws_workspaces WHERE id=?1",
                "DELETE FROM us_ledger WHERE workspace_id=?1",
                "DELETE FROM tr_poll_cursors WHERE key LIKE 'ws:session:%' AND json_extract(value,'$.workspaceId')=?1",
                "DELETE FROM ws_demo_workspaces WHERE workspace_id=?1",
            ] {
                tx.execute(sql, [&id]).map_err(backend)?;
            }
            tx.commit().map_err(backend)
        }).await
    }
}
