//! Polling framework for app triggers (Gmail new email, Shopify new product…): every
//! `interval`, call the poller with the stored cursor, start one run per new item, save the
//! new cursor. Cursors live in the store's key/value table, so restarts don't re-deliver.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use il_core::Item;
use serde_json::Value;

use crate::ctx::{TriggerCtx, TriggerHandle};
use crate::node::NodeError;
use crate::services::StartRun;

/// One poll: new items since `cursor`, and the cursor to store next.
#[async_trait]
pub trait Poller: Send + Sync + 'static {
    /// Fetches new items. Errors are logged and retried at the next interval.
    async fn poll(&self, ctx: &TriggerCtx, cursor: Option<Value>) -> Result<(Vec<Item>, Option<Value>), NodeError>;
}

/// Key under which a trigger node's cursor is stored.
pub fn cursor_key(flow_id: &str, node_id: &str) -> String {
    format!("tr:cursor:{flow_id}:{node_id}")
}

/// Starts polling; the handle stops it.
pub fn spawn_polling(
    ctx: TriggerCtx,
    trigger_type: &str,
    interval: Duration,
    poller: Arc<dyn Poller>,
) -> TriggerHandle {
    let cancel = ctx.cancel.clone();
    let trigger = trigger_type.to_string();
    let task = tokio::spawn(async move {
        let key = cursor_key(&ctx.flow_id, &ctx.node_id);
        loop {
            if let Err(e) = poll_once(&ctx, &trigger, &key, poller.as_ref()).await {
                tracing::warn!(flow_id = %ctx.flow_id, node_id = %ctx.node_id, error = %e, "poll failed");
            }
            tokio::select! {
                _ = ctx.cancel.cancelled() => break,
                _ = ctx.services.clock.sleep(interval) => {}
            }
        }
    });
    TriggerHandle::new(cancel, Some(task))
}

/// One poll cycle (public for tests).
pub async fn poll_once(ctx: &TriggerCtx, trigger: &str, key: &str, poller: &dyn Poller) -> Result<usize, NodeError> {
    let store = &ctx.services.store;
    let cursor = store.kv_get(key).await.map_err(|e| NodeError::Failed(e.to_string()))?;
    let (items, next) = poller.poll(ctx, cursor).await?;
    let n = items.len();
    for item in items {
        let req = StartRun {
            flow_id: ctx.flow_id.clone(),
            version: None,
            trigger: trigger.to_string(),
            items: vec![item],
            trigger_node: Some(ctx.node_id.clone()),
        };
        ctx.runner.start(req).await.map_err(|e| NodeError::Retryable(e.to_string()))?;
    }
    if let Some(next) = next {
        store.kv_put(key, &next).await.map_err(|e| NodeError::Failed(e.to_string()))?;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::testkit::{FakeRunner, FakeServices};

    /// Returns numbers greater than the cursor, up to 5.
    struct Counter;

    #[async_trait]
    impl Poller for Counter {
        async fn poll(
            &self,
            _ctx: &TriggerCtx,
            cursor: Option<Value>,
        ) -> Result<(Vec<Item>, Option<Value>), NodeError> {
            let from = cursor.and_then(|c| c.as_i64()).unwrap_or(0);
            let items: Vec<Item> = ((from + 1)..=(from + 2).min(5)).map(|n| Item::json(json!({ "n": n }))).collect();
            let next = items.last().map(|i| i.json["n"].clone());
            Ok((items, next))
        }
    }

    #[tokio::test]
    async fn polls_with_cursor_and_starts_one_run_per_item() {
        let fakes = FakeServices::default();
        let runner = Arc::new(FakeRunner::default());
        let ctx = TriggerCtx {
            flow_id: "flw".into(),
            workspace_id: "ws".into(),
            node_id: "n_mail".into(),
            settings: json!({}),
            services: fakes.services(),
            runner: runner.clone(),
            cancel: CancellationToken::new(),
        };
        let key = cursor_key("flw", "n_mail");
        assert_eq!(poll_once(&ctx, "app.x.new", &key, &Counter).await.unwrap(), 2);
        assert_eq!(poll_once(&ctx, "app.x.new", &key, &Counter).await.unwrap(), 2);
        assert_eq!(poll_once(&ctx, "app.x.new", &key, &Counter).await.unwrap(), 1);
        assert_eq!(poll_once(&ctx, "app.x.new", &key, &Counter).await.unwrap(), 0);
        let started = runner.started();
        assert_eq!(started.len(), 5);
        assert_eq!(started[4].items[0].json["n"], 5);
        assert_eq!(started[0].trigger_node.as_deref(), Some("n_mail"));
    }
}
