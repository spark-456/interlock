use chrono::Utc;
use il_core::{Approval, BuildEvent, Flow, NodeRun, Run, RunEvent};
use il_sdk::services::store::{Credential, SpendEntry, SpendScope, Store, StoreError};
use serde_json::json;

use super::{BuildRecord, ScheduleRecord, UserRecord, WorkspaceRecord};
use crate::db::SqliteStore;

fn setup() -> (tempfile::TempDir, SqliteStore) {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(dir.path().join("store.sqlite")).unwrap().with_master_key([9; 32]);
    (dir, store)
}

fn flow() -> Flow {
    serde_json::from_value(json!({"id":"fl_1","workspaceId":"ws_1","version":0,"name":"Flow",
        "kind":"automation","nodes":[],"edges":[]}))
    .unwrap()
}

fn run() -> Run {
    serde_json::from_value(json!({"id":"rn_1","flowId":"fl_1","workspaceId":"ws_1",
        "flowVersion":1,"status":"running","trigger":"test","startedAt":Utc::now(),
        "finishedAt":null,"costUsd":0,"error":null}))
    .unwrap()
}

#[tokio::test]
async fn b01_flow_versions_and_active_recovery() {
    let (_dir, store) = setup();
    let mut flow = flow();
    assert_eq!(store.put_flow(&flow).await.unwrap(), 1);
    flow.name = "Changed".into();
    assert_eq!(store.put_flow(&flow).await.unwrap(), 2);
    assert_eq!(store.get_flow("fl_1").await.unwrap().unwrap().version, 2);
    assert_eq!(store.flow_version("fl_1", 1).await.unwrap().unwrap().name, "Flow");
    assert_eq!(store.list_flows("ws_1").await.unwrap().len(), 1);
    store.set_flow_active("fl_1", true).await.unwrap();
    assert!(store.is_flow_active("fl_1").await.unwrap());
    assert_eq!(store.active_flows().await.unwrap().len(), 1);
    store.delete_flow("fl_1").await.unwrap();
    assert!(store.get_flow("fl_1").await.unwrap().is_none());
}

#[tokio::test]
async fn b01_runs_node_runs_and_approvals() {
    let (_dir, store) = setup();
    let run = run();
    store.put_run(&run).await.unwrap();
    assert_eq!(store.get_run("rn_1").await.unwrap(), Some(run.clone()));
    assert_eq!(store.list_runs("fl_1", 1).await.unwrap(), vec![run.clone()]);
    assert_eq!(store.recoverable_runs().await.unwrap(), vec![run]);
    let node: NodeRun = serde_json::from_value(json!({"runId":"rn_1","nodeId":"n_1",
        "status":"succeeded","attempt":1,"input":[],"output":{},"error":null,"ms":1,"costUsd":0}))
    .unwrap();
    store.put_node_run(&node).await.unwrap();
    assert_eq!(store.node_runs("rn_1").await.unwrap(), vec![node]);
    let approval: Approval = serde_json::from_value(json!({"id":"ap_1","workspaceId":"ws_1",
        "flowId":"fl_1","runId":"rn_1","nodeId":"n_1","title":"Approve","message":"Okay?",
        "payload":{},"status":"pending","channels":[],"createdAt":Utc::now(),
        "decidedAt":null,"decidedBy":null}))
    .unwrap();
    store.put_approval(&approval).await.unwrap();
    assert_eq!(store.get_approval("ap_1").await.unwrap(), Some(approval.clone()));
    assert_eq!(store.list_approvals("ws_1", true).await.unwrap(), vec![approval]);
    let mut due = store.get_approval("ap_1").await.unwrap().unwrap();
    due.expires_at = Some(Utc::now() - chrono::Duration::minutes(1));
    due.on_timeout = Some(il_core::ApprovalTimeout::Reject);
    store.put_approval(&due).await.unwrap();
    assert_eq!(store.due_approvals(Utc::now()).await.unwrap(), vec![due.clone()]);
    due.status = il_core::ApprovalStatus::Rejected;
    assert!(store.transition_approval(&due).await.unwrap());
    assert!(!store.transition_approval(&due).await.unwrap());
    assert!(store.due_approvals(Utc::now()).await.unwrap().is_empty());
}

#[tokio::test]
async fn b01_events_cursor_and_order_are_durable() {
    let (dir, store) = setup();
    let run_event: RunEvent = serde_json::from_value(json!({"seq":1,"ts":Utc::now(),
        "runId":"rn_1","type":"run.started","data":{"flowId":"fl_1","trigger":"test"}}))
    .unwrap();
    store.append_run_event(&run_event).await.unwrap();
    assert!(matches!(store.append_run_event(&run_event).await, Err(StoreError::Conflict(_))));
    let build_event: BuildEvent = serde_json::from_value(json!({"seq":1,"ts":Utc::now(),
        "buildId":"bd_1","type":"build.log","data":{"text":"started"}}))
    .unwrap();
    store.append_build_event(&build_event).await.unwrap();
    drop(store);
    let reopened = SqliteStore::open(dir.path().join("store.sqlite")).unwrap();
    assert_eq!(reopened.run_events("rn_1", 0).await.unwrap(), vec![run_event]);
    assert!(reopened.run_events("rn_1", 1).await.unwrap().is_empty());
    assert_eq!(reopened.build_events("bd_1", 0).await.unwrap(), vec![build_event]);
}

#[tokio::test]
async fn b01_credentials_are_encrypted_and_workspace_scoped() {
    let (_dir, store) = setup();
    let credential = Credential {
        id: "cred_1".into(),
        app: "example".into(),
        secret: json!({"token":"top secret"}),
        meta: json!({"account":"test"}),
    };
    store.put_credential("ws_1", &credential).await.unwrap();
    assert_eq!(store.credential("ws_1", "cred_1").await.unwrap().unwrap().secret, credential.secret);
    assert!(store.credential("ws_2", "cred_1").await.unwrap().is_none());
    let listed = store.credentials("ws_1").await.unwrap();
    assert_eq!((listed.len(), listed[0].id.as_str(), listed[0].meta.clone()), (1, "cred_1", credential.meta.clone()));
    assert!(store.credentials("ws_2").await.unwrap().is_empty());
    let conn = store.conn.lock().unwrap();
    let ciphertext: Vec<u8> = conn.query_row("SELECT ciphertext FROM cn_credentials", [], |r| r.get(0)).unwrap();
    assert!(!String::from_utf8_lossy(&ciphertext).contains("top secret"));
}

#[tokio::test]
async fn b01_poll_state_and_spend_scopes() {
    let (_dir, store) = setup();
    store.kv_put("poll:gmail", &json!({"cursor":42})).await.unwrap();
    assert_eq!(store.kv_get("poll:gmail").await.unwrap(), Some(json!({"cursor":42})));
    let entry = SpendEntry {
        workspace_id: "ws_1".into(),
        flow_id: Some("fl_1".into()),
        run_id: Some("rn_1".into()),
        build_id: None,
        role: "fast".into(),
        cost_usd: 0.25,
        at: Utc::now(),
    };
    store.record_spend(&entry).await.unwrap();
    assert_eq!(store.spend(&SpendScope::Run("rn_1".into())).await.unwrap(), 0.25);
    assert_eq!(store.spend(&SpendScope::WorkspaceSince("ws_2".into(), entry.at)).await.unwrap(), 0.0);
}

#[tokio::test]
async fn b01_workspaces_users_members_and_hashed_api_keys() {
    let (_dir, store) = setup();
    let ws = WorkspaceRecord { id: "ws_1".into(), name: "Team".into(), created_at: Utc::now() };
    let user = UserRecord {
        id: "usr_1".into(),
        email: "example@example.com".into(),
        name: "Example".into(),
        created_at: Utc::now(),
    };
    store.put_workspace(&ws).await.unwrap();
    store.put_user(&user).await.unwrap();
    store.put_member("ws_1", "usr_1", "owner").await.unwrap();
    assert_eq!(store.get_workspace("ws_1").await.unwrap(), Some(ws));
    assert_eq!(store.get_user("usr_1").await.unwrap(), Some(user));
    assert_eq!(store.member_role("ws_1", "usr_1").await.unwrap().as_deref(), Some("owner"));
    store.create_api_key("key_1", "ws_1", "SDK", "il_plaintext_key").await.unwrap();
    assert_eq!(store.list_api_keys("ws_1").await.unwrap()[0].name, "SDK");
    assert_eq!(store.workspace_for_api_key("il_plaintext_key").await.unwrap().as_deref(), Some("ws_1"));
    store.revoke_api_key("key_1").await.unwrap();
    assert!(store.workspace_for_api_key("il_plaintext_key").await.unwrap().is_none());
}

#[tokio::test]
async fn b01_schedules_webhooks_and_builds() {
    let (_dir, store) = setup();
    let schedule = ScheduleRecord {
        id: "sch_1".into(),
        flow_id: "fl_1".into(),
        cron: "0 * * * *".into(),
        timezone: "UTC".into(),
        enabled: true,
    };
    store.put_schedule(&schedule).await.unwrap();
    assert_eq!(store.enabled_schedules().await.unwrap(), vec![schedule]);
    store.put_webhook("hook_1", "fl_1", "one-time-token", true).await.unwrap();
    assert_eq!(store.webhook_for_token("one-time-token").await.unwrap().unwrap().flow_id, "fl_1");
    assert!(store.webhook_for_token("wrong").await.unwrap().is_none());
    let build = BuildRecord {
        id: "bd_1".into(),
        workspace_id: "ws_1".into(),
        status: "running".into(),
        created_at: Utc::now(),
        body: json!({"goal":"hello"}),
    };
    store.put_build(&build).await.unwrap();
    assert_eq!(store.get_build("bd_1").await.unwrap(), Some(build));
}
