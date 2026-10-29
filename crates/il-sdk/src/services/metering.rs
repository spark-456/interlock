//! Workspace-aware usage ledger around a model gateway.

use std::{future::Future, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt;
use il_core::FileRef;

use super::models::{
    ChatDelta, ChatRequest, ChatResponse, ChatStream, ModelError, ModelGateway, ModelRole, Transcript,
};
use super::store::SpendEntry;
use super::{Clock, Store};

/// Attribution for one build, edit, or run.
#[derive(Clone)]
pub struct ModelCallContext {
    pub workspace_id: String,
    pub flow_id: Option<String>,
    pub run_id: Option<String>,
    pub build_id: Option<String>,
}

tokio::task_local! { static MODEL_CONTEXT: ModelCallContext; }

/// Runs a future with workspace attribution for model calls it makes.
pub async fn with_model_context<F: Future>(context: ModelCallContext, future: F) -> F::Output {
    MODEL_CONTEXT.scope(context, future).await
}

/// Records priced model calls while delegating to the configured gateway.
pub struct MeteredGateway {
    inner: Arc<dyn ModelGateway>,
    store: Arc<dyn Store>,
    clock: Arc<dyn Clock>,
}

impl MeteredGateway {
    /// Wraps a gateway and its ledger and clock.
    pub fn new(inner: Arc<dyn ModelGateway>, store: Arc<dyn Store>, clock: Arc<dyn Clock>) -> Self {
        Self { inner, store, clock }
    }
}

fn role_name(role: ModelRole) -> &'static str {
    match role {
        ModelRole::Architect => "architect",
        ModelRole::Agent => "agent",
        ModelRole::Fast => "fast",
    }
}

async fn record(
    store: &dyn Store,
    clock: &dyn Clock,
    context: Option<ModelCallContext>,
    role: ModelRole,
    cost_usd: f64,
) {
    let Some(context) = context else { return };
    if !cost_usd.is_finite() || cost_usd <= 0.0 {
        return;
    }
    let entry = SpendEntry {
        workspace_id: context.workspace_id,
        flow_id: context.flow_id,
        run_id: context.run_id,
        build_id: context.build_id,
        role: role_name(role).into(),
        cost_usd,
        at: clock.now(),
    };
    if let Err(error) = store.record_spend(&entry).await {
        tracing::error!(error = %error, "model spend could not be recorded");
    }
}

#[async_trait]
impl ModelGateway for MeteredGateway {
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        let response = self.inner.chat(role, req).await?;
        record(
            self.store.as_ref(),
            self.clock.as_ref(),
            MODEL_CONTEXT.try_with(Clone::clone).ok(),
            role,
            response.usage.cost_usd,
        )
        .await;
        Ok(response)
    }
    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let stream = self.inner.chat_stream(role, req).await?;
        let (store, clock, context) =
            (self.store.clone(), self.clock.clone(), MODEL_CONTEXT.try_with(Clone::clone).ok());
        Ok(Box::pin(stream.then(move |delta| {
            let (store, clock, context) = (store.clone(), clock.clone(), context.clone());
            async move {
                if let Ok(ChatDelta::Done(usage)) = &delta {
                    record(store.as_ref(), clock.as_ref(), context, role, usage.cost_usd).await;
                }
                delta
            }
        })))
    }
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        self.inner.embed(texts).await
    }
    async fn transcribe(&self, file: &FileRef, bytes: &[u8]) -> Result<Transcript, ModelError> {
        self.inner.transcribe(file, bytes).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        services::{
            models::{ChatResponse, Usage},
            store::SpendScope,
        },
        testkit::FakeServices,
    };

    #[tokio::test]
    async fn u02_gateway_records_each_call_by_role_and_workspace() {
        let fakes = FakeServices::default();
        for role in [ModelRole::Architect, ModelRole::Agent] {
            fakes
                .models
                .push(ChatResponse { usage: Usage { cost_usd: 0.25, ..Default::default() }, ..Default::default() });
            let gateway = MeteredGateway::new(fakes.models.clone(), fakes.store.clone(), fakes.clock.clone());
            with_model_context(
                ModelCallContext {
                    workspace_id: "ws_one".into(),
                    flow_id: Some("flow".into()),
                    run_id: None,
                    build_id: Some("build".into()),
                },
                gateway.chat(role, ChatRequest::default()),
            )
            .await
            .unwrap();
        }
        let entries = fakes.store.spend_entries("ws_one", fakes.clock.now()).await.unwrap();
        assert_eq!(entries.iter().map(|e| e.role.as_str()).collect::<Vec<_>>(), ["architect", "agent"]);
        assert_eq!(
            fakes.store.spend(&SpendScope::WorkspaceSince("ws_one".into(), fakes.clock.now())).await.unwrap(),
            0.5
        );
    }
}
