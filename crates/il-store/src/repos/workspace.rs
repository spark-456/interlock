//! Deleting a workspace and everything recorded for it.

use il_sdk::services::store::StoreResult;

use crate::db::{SqliteStore, backend};

impl SqliteStore {
    /// Removes a workspace's data in one transaction, after its files have been purged.
    pub async fn delete_workspace(&self, workspace_id: &str) -> StoreResult<()> {
        let id = workspace_id.to_owned();
        self.with_conn(move |c| {
            let tx = c.transaction().map_err(backend)?;
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
            ] {
                tx.execute(sql, [&id]).map_err(backend)?;
            }
            tx.commit().map_err(backend)
        }).await
    }
}
