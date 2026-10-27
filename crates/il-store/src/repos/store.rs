use async_trait::async_trait;
use il_core::{Approval, BuildEvent, Flow, NodeRun, Run, RunEvent};
use il_sdk::services::store::{Credential, SpendEntry, SpendScope, Store, StoreError, StoreResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    db::{SqliteStore, backend},
    secrets,
};

fn json<T: Serialize>(value: &T) -> StoreResult<String> {
    serde_json::to_string(value).map_err(backend)
}

fn parse<T: DeserializeOwned>(value: String) -> StoreResult<T> {
    serde_json::from_str(&value).map_err(backend)
}

fn get<T: DeserializeOwned>(conn: &Connection, sql: &str, id: &str) -> StoreResult<Option<T>> {
    let body: Option<String> = conn.query_row(sql, [id], |r| r.get(0)).optional().map_err(backend)?;
    body.map(parse).transpose()
}

fn list<T: DeserializeOwned>(conn: &Connection, sql: &str, id: &str) -> StoreResult<Vec<T>> {
    let mut stmt = conn.prepare(sql).map_err(backend)?;
    let rows = stmt.query_map([id], |r| r.get::<_, String>(0)).map_err(backend)?;
    rows.map(|row| parse(row.map_err(backend)?)).collect()
}

#[async_trait]
impl Store for SqliteStore {
    async fn get_flow(&self, id: &str) -> StoreResult<Option<Flow>> {
        let id = id.to_owned();
        self.with_conn(move |c| get(c, "SELECT body FROM fl_flows WHERE id=?1", &id)).await
    }

    async fn put_flow(&self, flow: &Flow) -> StoreResult<u32> {
        let mut flow = flow.clone();
        self.with_conn(move |c| {
            let tx = c.transaction().map_err(backend)?;
            let version: u32 = tx.query_row(
                "SELECT version FROM fl_flows WHERE id=?1", [&flow.id], |r| r.get(0)
            ).optional().map_err(backend)?.unwrap_or(0) + 1;
            flow.version = version;
            let body = json(&flow)?;
            tx.execute("INSERT INTO fl_flows (id, workspace_id, version, body) VALUES (?1,?2,?3,?4)
                        ON CONFLICT(id) DO UPDATE SET workspace_id=excluded.workspace_id, version=excluded.version, body=excluded.body",
                params![flow.id, flow.workspace_id, version, body]).map_err(backend)?;
            tx.execute("INSERT INTO fl_versions (flow_id, version, body) VALUES (?1,?2,?3)",
                params![flow.id, version, body]).map_err(backend)?;
            tx.commit().map_err(backend)?;
            Ok(version)
        }).await
    }

    async fn list_flows(&self, workspace_id: &str) -> StoreResult<Vec<Flow>> {
        let id = workspace_id.to_owned();
        self.with_conn(move |c| list(c, "SELECT body FROM fl_flows WHERE workspace_id=?1 ORDER BY id", &id)).await
    }

    async fn delete_flow(&self, id: &str) -> StoreResult<()> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            let tx = c.transaction().map_err(backend)?;
            tx.execute("DELETE FROM fl_versions WHERE flow_id=?1", [&id]).map_err(backend)?;
            tx.execute("DELETE FROM fl_flows WHERE id=?1", [&id]).map_err(backend)?;
            tx.commit().map_err(backend)?;
            Ok(())
        })
        .await
    }

    async fn flow_version(&self, id: &str, version: u32) -> StoreResult<Option<Flow>> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            let body: Option<String> = c
                .query_row("SELECT body FROM fl_versions WHERE flow_id=?1 AND version=?2", params![id, version], |r| {
                    r.get(0)
                })
                .optional()
                .map_err(backend)?;
            body.map(parse).transpose()
        })
        .await
    }

    async fn set_flow_active(&self, id: &str, active: bool) -> StoreResult<()> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            if c.execute("UPDATE fl_flows SET active=?2 WHERE id=?1", params![id, active]).map_err(backend)? == 0 {
                return Err(StoreError::NotFound(id));
            }
            Ok(())
        })
        .await
    }

    async fn is_flow_active(&self, id: &str) -> StoreResult<bool> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            Ok(c.query_row("SELECT active FROM fl_flows WHERE id=?1", [&id], |r| r.get(0))
                .optional()
                .map_err(backend)?
                .unwrap_or(false))
        })
        .await
    }

    async fn active_flows(&self) -> StoreResult<Vec<Flow>> {
        SqliteStore::active_flows(self).await
    }

    async fn put_run(&self, run: &Run) -> StoreResult<()> {
        let run = run.clone();
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO rn_runs (id, flow_id, workspace_id, status, started_at, body) VALUES (?1,?2,?3,?4,?5,?6)
                       ON CONFLICT(id) DO UPDATE SET status=excluded.status, body=excluded.body",
                params![
                    run.id,
                    run.flow_id,
                    run.workspace_id,
                    format!("{:?}", run.status),
                    run.started_at.to_rfc3339(),
                    json(&run)?
                ],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    async fn get_run(&self, id: &str) -> StoreResult<Option<Run>> {
        let id = id.to_owned();
        self.with_conn(move |c| get(c, "SELECT body FROM rn_runs WHERE id=?1", &id)).await
    }

    async fn list_runs(&self, flow_id: &str, limit: usize) -> StoreResult<Vec<Run>> {
        let id = flow_id.to_owned();
        self.with_conn(move |c| {
            let mut stmt = c
                .prepare("SELECT body FROM rn_runs WHERE flow_id=?1 ORDER BY started_at DESC, id DESC LIMIT ?2")
                .map_err(backend)?;
            let rows = stmt.query_map(params![id, limit as i64], |r| r.get::<_, String>(0)).map_err(backend)?;
            rows.map(|row| parse(row.map_err(backend)?)).collect()
        })
        .await
    }

    async fn recoverable_runs(&self) -> StoreResult<Vec<Run>> {
        SqliteStore::recoverable_runs(self).await
    }

    async fn put_node_run(&self, nr: &NodeRun) -> StoreResult<()> {
        let nr = nr.clone();
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO rn_node_runs (run_id,node_id,body) VALUES (?1,?2,?3)
                       ON CONFLICT(run_id,node_id) DO UPDATE SET body=excluded.body",
                params![nr.run_id, nr.node_id, json(&nr)?],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    async fn node_runs(&self, run_id: &str) -> StoreResult<Vec<NodeRun>> {
        let id = run_id.to_owned();
        self.with_conn(move |c| list(c, "SELECT body FROM rn_node_runs WHERE run_id=?1 ORDER BY node_id", &id)).await
    }

    async fn append_run_event(&self, event: &RunEvent) -> StoreResult<()> {
        let event = event.clone();
        self.with_conn(move |c| append_event(c, "rn_events", "run_id", &event.run_id, event.seq, json(&event)?)).await
    }

    async fn run_events(&self, run_id: &str, after: u64) -> StoreResult<Vec<RunEvent>> {
        let id = run_id.to_owned();
        self.with_conn(move |c| events(c, "rn_events", "run_id", &id, after)).await
    }

    async fn append_build_event(&self, event: &BuildEvent) -> StoreResult<()> {
        let event = event.clone();
        self.with_conn(move |c| append_event(c, "bd_events", "build_id", &event.build_id, event.seq, json(&event)?))
            .await
    }

    async fn build_events(&self, build_id: &str, after: u64) -> StoreResult<Vec<BuildEvent>> {
        let id = build_id.to_owned();
        self.with_conn(move |c| events(c, "bd_events", "build_id", &id, after)).await
    }

    async fn put_approval(&self, approval: &Approval) -> StoreResult<()> {
        let approval = approval.clone();
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO rn_approvals (id,workspace_id,status,created_at,body) VALUES (?1,?2,?3,?4,?5)
                       ON CONFLICT(id) DO UPDATE SET status=excluded.status, body=excluded.body",
                params![
                    approval.id,
                    approval.workspace_id,
                    format!("{:?}", approval.status),
                    approval.created_at.to_rfc3339(),
                    json(&approval)?
                ],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    async fn get_approval(&self, id: &str) -> StoreResult<Option<Approval>> {
        let id = id.to_owned();
        self.with_conn(move |c| get(c, "SELECT body FROM rn_approvals WHERE id=?1", &id)).await
    }

    async fn list_approvals(&self, workspace_id: &str, pending_only: bool) -> StoreResult<Vec<Approval>> {
        let id = workspace_id.to_owned();
        self.with_conn(move |c| {
            let sql = if pending_only {
                "SELECT body FROM rn_approvals WHERE workspace_id=?1 AND status='Pending' ORDER BY created_at DESC"
            } else {
                "SELECT body FROM rn_approvals WHERE workspace_id=?1 ORDER BY created_at DESC"
            };
            list(c, sql, &id)
        })
        .await
    }

    async fn credential(&self, workspace_id: &str, id: &str) -> StoreResult<Option<Credential>> {
        let key = self.key;
        let workspace_id = workspace_id.to_owned();
        let id = id.to_owned();
        self.with_conn(move |c| {
            let row: Option<(String, Vec<u8>, Vec<u8>, String)> = c
                .query_row(
                    "SELECT app,nonce,ciphertext,meta FROM cn_credentials WHERE workspace_id=?1 AND id=?2",
                    params![workspace_id, id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()
                .map_err(backend)?;
            row.map(|(app, nonce, ciphertext, meta)| {
                let key = key.ok_or_else(|| StoreError::Backend("IL_MASTER_KEY is required for credentials".into()))?;
                let plain = secrets::open(&key, &workspace_id, &id, &nonce, &ciphertext)?;
                Ok(Credential { id, app, secret: serde_json::from_slice(&plain).map_err(backend)?, meta: parse(meta)? })
            })
            .transpose()
        })
        .await
    }

    async fn kv_get(&self, key: &str) -> StoreResult<Option<Value>> {
        let key = key.to_owned();
        self.with_conn(move |c| get(c, "SELECT value FROM tr_poll_cursors WHERE key=?1", &key)).await
    }

    async fn kv_put(&self, key: &str, value: &Value) -> StoreResult<()> {
        let key = key.to_owned();
        let body = json(value)?;
        self.with_conn(move |c| {
            c.execute("INSERT INTO tr_poll_cursors (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![key, body]).map_err(backend)?;
            Ok(())
        }).await
    }

    async fn record_spend(&self, entry: &SpendEntry) -> StoreResult<()> {
        let e = entry.clone();
        self.with_conn(move |c| {
            c.execute("INSERT INTO us_ledger (workspace_id,flow_id,run_id,build_id,role,cost_usd,at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![e.workspace_id, e.flow_id, e.run_id, e.build_id, e.role, e.cost_usd, e.at.to_rfc3339()]).map_err(backend)?;
            Ok(())
        }).await
    }

    async fn spend(&self, scope: &SpendScope) -> StoreResult<f64> {
        let scope = scope.clone();
        self.with_conn(move |c| {
            let (sql, a, b) = match scope {
                SpendScope::Run(id) => ("SELECT COALESCE(sum(cost_usd),0) FROM us_ledger WHERE run_id=?1", id, None),
                SpendScope::FlowSince(id, at) => (
                    "SELECT COALESCE(sum(cost_usd),0) FROM us_ledger WHERE flow_id=?1 AND at>=?2",
                    id,
                    Some(at.to_rfc3339()),
                ),
                SpendScope::WorkspaceSince(id, at) => (
                    "SELECT COALESCE(sum(cost_usd),0) FROM us_ledger WHERE workspace_id=?1 AND at>=?2",
                    id,
                    Some(at.to_rfc3339()),
                ),
                SpendScope::GlobalSince(at) => {
                    ("SELECT COALESCE(sum(cost_usd),0) FROM us_ledger WHERE at>=?1", at.to_rfc3339(), None)
                }
            };
            if let Some(b) = b {
                c.query_row(sql, params![a, b], |r| r.get(0)).map_err(backend)
            } else {
                c.query_row(sql, [a], |r| r.get(0)).map_err(backend)
            }
        })
        .await
    }
}

fn append_event(
    conn: &mut Connection,
    table: &str,
    stream_col: &str,
    id: &str,
    seq: u64,
    body: String,
) -> StoreResult<()> {
    let tx = conn.transaction().map_err(backend)?;
    let max: Option<u64> = tx
        .query_row(&format!("SELECT max(seq) FROM {table} WHERE {stream_col}=?1"), [id], |r| r.get(0))
        .map_err(backend)?;
    if max.is_some_and(|last| seq <= last) {
        return Err(StoreError::Conflict(format!("seq {seq} out of order")));
    }
    tx.execute(&format!("INSERT INTO {table} ({stream_col},seq,body) VALUES (?1,?2,?3)"), params![id, seq, body])
        .map_err(backend)?;
    tx.commit().map_err(backend)
}

fn events<T: DeserializeOwned>(
    conn: &Connection,
    table: &str,
    stream_col: &str,
    id: &str,
    after: u64,
) -> StoreResult<Vec<T>> {
    let mut stmt = conn
        .prepare(&format!("SELECT body FROM {table} WHERE {stream_col}=?1 AND seq>?2 ORDER BY seq"))
        .map_err(backend)?;
    let rows = stmt.query_map(params![id, after], |r| r.get::<_, String>(0)).map_err(backend)?;
    rows.map(|row| parse(row.map_err(backend)?)).collect()
}
