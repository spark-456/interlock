//! `ai.agent` — Agent.
//!
//! Calls the configured knowledge bases and tools through SDK traits.

use super::common;
use il_sdk::prelude::*;
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelRole, ToolSpec};

const PROMPT: &str = include_str!("../../../../prompts/nodes/agent.md");

/// Agent node.
#[derive(Debug, Default)]
pub struct AiAgent;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("ai.agent", Category::Ai, "Agent", "An assistant that answers using knowledge bases and tools in a loop, returning the answer, sources and activity.")
        .icon("bot")
    .status(NodeStatus::Ready)
        .cost(CostHint::Medium)
        .credential(CredentialSpec { optional: true, ..cred("api", "openapi", CredentialKind::ApiKey, "API key", &[]) })
        .settings_schema(json!({
            "type": "object",
            "required": [
                "instructions"
            ],
            "properties": {
                "instructions": {
                    "type": "string",
                    "title": "Instructions",
                    "x-widget": "longtext"
                },
                "input": {
                    "type": "string",
                    "title": "Input",
                    "default": "{{ item.message }}",
                    "x-widget": "expression"
                },
                "knowledgeBases": {
                    "type": "array",
                    "title": "Knowledge",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "tools": {
                    "type": "array",
                    "title": "Tools",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "apiTools": {
                    "type": "object",
                    "title": "API tools",
                    "description": "Operations from an uploaded OpenAPI spec, available to this agent without an execution edge.",
                    "required": ["spec", "operations"],
                    "properties": {
                        "spec": {"type": "string", "title": "OpenAPI spec", "description": "URL or file id."},
                        "operations": {"type": "array", "title": "Operations", "items": {"type": "string"}}
                    }
                },
                "maxSteps": {
                    "type": "integer",
                    "title": "Max steps",
                    "default": 8,
                    "minimum": 1,
                    "maximum": 20
                },
                "model": {
                    "type": "string",
                    "title": "Model",
                    "enum": [
                        "architect",
                        "agent",
                        "fast"
                    ],
                    "default": "agent",
                    "x-widget": "model-role"
                }
            }
        }))
        .example("Support agent", json!({
            "instructions": "You are Larkspur's support assistant. Answer from the help center; cite sources.",
            "knowledgeBases": [
                "kb_help_center"
            ],
            "tools": [
                "get_invoice"
            ]
        }))
});

#[async_trait]
impl NodeExecutor for AiAgent {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let bases: Vec<&str> =
            ctx.settings["knowledgeBases"].as_array().into_iter().flatten().filter_map(|v| v.as_str()).collect();
        let mut selected: Vec<&str> =
            ctx.settings["tools"].as_array().into_iter().flatten().filter_map(|v| v.as_str()).collect();
        selected
            .extend(ctx.settings["apiTools"]["operations"].as_array().into_iter().flatten().filter_map(|v| v.as_str()));
        let mut tools = Vec::new();
        let mut activity = Vec::new();
        let names = if bases.is_empty() {
            Vec::new()
        } else {
            ctx.knowledge().list_bases(&ctx.run.workspace_id).await.unwrap_or_default()
        };
        for base in &bases {
            let name = names.iter().find(|b| b["id"] == *base).and_then(|b| b["name"].as_str()).unwrap_or(base);
            tools.push(ToolSpec {
                name: format!("search_{base}"),
                // The id means nothing to the model; the name and a nudge make it search before giving up.
                description: format!(
                    "Search the \"{name}\" knowledge base (the documents and help pages you were given). \
                     Search it before answering a question, and before saying you don't know."
                ),
                parameters: json!({"type":"object","required":["query"],"properties":{"query":{"type":"string"}}}),
            });
        }
        for name in &selected {
            if let Some(tool) = ctx.tools().iter().find(|tool| tool.name() == *name) {
                tools.push(ToolSpec {
                    name: tool.name().into(),
                    description: tool.description().into(),
                    parameters: tool.schema(),
                });
            } else {
                activity.push(json!({"tool":name,"warning":format!("tool {name} is unavailable")}));
            }
        }
        let mut messages = vec![
            ChatMessage::system(format!(
                "{}\n{}",
                PROMPT.trim(),
                ctx.settings["instructions"].as_str().unwrap_or_default()
            )),
            ChatMessage::user(common::input_text(&ctx.settings, &input)),
        ];
        let max_steps = ctx.settings["maxSteps"].as_u64().unwrap_or(8).clamp(1, 20);
        let mut sources = Vec::new();
        for _ in 0..max_steps {
            let response = ctx
                .models()
                .chat(
                    common::role(&ctx.settings, ModelRole::Agent)?,
                    ChatRequest { messages: messages.clone(), tools: tools.clone(), ..Default::default() },
                )
                .await
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            if response.tool_calls.is_empty() {
                let mut item = input.main().first().cloned().unwrap_or_default();
                if !item.json.is_object() {
                    item.json = json!({});
                }
                item.json["answer"] = json!(response.content);
                item.json["sources"] = json!(sources);
                item.json["activity"] = json!(activity);
                return Ok(NodeOutput::out(vec![item]));
            }
            let mut assistant = ChatMessage::assistant(response.content);
            assistant.tool_calls = response.tool_calls.clone();
            messages.push(assistant);
            for call in response.tool_calls {
                let result = if let Some(base) = bases.iter().find(|base| call.name == format!("search_{base}")) {
                    // Like a failing tool, a failed search is the model's to explain, not a failed step.
                    let query = call.arguments["query"].as_str().filter(|q| !q.trim().is_empty());
                    let found = match query {
                        Some(query) => ctx.knowledge().search(base, query, 5).await.map_err(|e| e.to_string()),
                        None => Err("the search needs a query".into()),
                    };
                    match found {
                        Ok(hits) => {
                            // One entry per document: several passages often come from the same one.
                            for hit in &hits {
                                if !sources.iter().any(|s: &Value| s["sourceId"] == hit.source_id) {
                                    sources.push(json!({"sourceId":hit.source_id,"title":hit.title,"section":hit.section,"url":hit.url}));
                                }
                            }
                            json!(hits)
                        }
                        Err(error) => {
                            tracing::warn!(run_id = %ctx.run.run_id, node_id = %ctx.run.node_id, %error, "knowledge search failed");
                            json!({ "error": error })
                        }
                    }
                } else {
                    let Some(tool) =
                        ctx.tools().iter().find(|tool| tool.name() == call.name && selected.contains(&tool.name()))
                    else {
                        let warning = format!("tool {} is unavailable", call.name);
                        activity.push(json!({"tool":call.name,"warning":warning}));
                        messages.push(ChatMessage::tool(call.id, json!({"error":warning}).to_string()));
                        continue;
                    };
                    // The model can tell the person the system is unavailable; failing the step shows a raw error.
                    tool.invoke(call.arguments).await.unwrap_or_else(|e| json!({"error": e.to_string()}))
                };
                activity.push(json!({"tool":call.name,"result":result}));
                messages.push(ChatMessage::tool(call.id, result.to_string()));
            }
        }
        Err(NodeError::Failed(format!("agent exceeded {max_steps} steps")))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiAgent)) }

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use il_sdk::services::knowledge::SearchHit;
    use il_sdk::services::{AgentTool, ToolError};
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiAgent);
    }

    #[tokio::test]
    async fn b07_agent_searches_and_returns_sources() {
        let harness = NodeHarness::new(super::AiAgent)
            .settings(json!({"instructions":"Answer with citations","input":"{{ item.question }}","knowledgeBases":["help"],"maxSteps":3}))
            .input_json(vec![json!({"question":"Refund policy?"})]);
        harness.fakes.knowledge.hits.lock().unwrap().push(SearchHit {
            text: "Refunds within 30 days".into(),
            score: 0.9,
            source_id: "src1".into(),
            title: "Policy".into(),
            section: None,
            url: None,
        });
        harness
            .fakes
            .models
            .push_tool_call("search_help", json!({"query":"refund"}))
            .push_text("Refunds within 30 days.");
        let output = harness.run().await.unwrap();
        assert_eq!(output.ports["out"][0].json["answer"], "Refunds within 30 days.");
        assert_eq!(output.ports["out"][0].json["sources"][0]["sourceId"], "src1");
        assert_eq!(harness.fakes.models.calls()[1].1.messages.len(), 4);
    }

    #[tokio::test]
    async fn q21_a_search_without_a_query_is_reported_to_the_model() {
        let harness = NodeHarness::new(super::AiAgent)
            .settings(json!({"instructions":"Answer","knowledgeBases":["help"],"maxSteps":3}));
        harness.fakes.models.push_tool_call("search_help", json!({})).push_text("What would you like to know?");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["answer"], "What would you like to know?");
        assert_eq!(out.ports["out"][0].json["activity"][0]["result"]["error"], "the search needs a query");
    }

    #[tokio::test]
    async fn b07_agent_honors_step_limit() {
        let harness = NodeHarness::new(super::AiAgent)
            .settings(json!({"instructions":"Use tools","knowledgeBases":["help"],"maxSteps":1}));
        harness.fakes.models.push_tool_call("search_help", json!({"query":"x"}));
        assert!(harness.run().await.unwrap_err().to_string().contains("exceeded 1 steps"));
    }

    struct InvoiceTool;

    #[async_trait]
    impl AgentTool for InvoiceTool {
        fn name(&self) -> &str {
            "get_invoice"
        }
        fn description(&self) -> &str {
            "Look up an invoice"
        }
        fn schema(&self) -> serde_json::Value {
            json!({"type":"object"})
        }
        async fn invoke(&self, _args: serde_json::Value) -> Result<serde_json::Value, ToolError> {
            Ok(json!({"status":"paid"}))
        }
    }

    #[tokio::test]
    async fn b07_agent_calls_selected_tool() {
        let mut harness = NodeHarness::new(super::AiAgent)
            .settings(json!({"instructions":"Check invoices","tools":["get_invoice"],"maxSteps":3}));
        harness.fakes.tools.push(std::sync::Arc::new(InvoiceTool));
        harness.fakes.models.push_tool_call("get_invoice", json!({"id":"inv1"})).push_text("Invoice paid.");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["activity"][0]["result"]["status"], "paid");
        assert_eq!(harness.fakes.models.calls()[0].1.tools[0].name, "get_invoice");
    }

    struct DownTool;

    #[async_trait]
    impl AgentTool for DownTool {
        fn name(&self) -> &str {
            "get_invoice"
        }
        fn description(&self) -> &str {
            "Look up an invoice"
        }
        fn schema(&self) -> serde_json::Value {
            json!({"type":"object"})
        }
        async fn invoke(&self, _args: serde_json::Value) -> Result<serde_json::Value, ToolError> {
            Err(ToolError::Failed("billing system unreachable".into()))
        }
    }

    #[tokio::test]
    async fn q20_a_failing_tool_is_reported_to_the_model_not_the_person() {
        let mut harness = NodeHarness::new(super::AiAgent)
            .settings(json!({"instructions":"Check invoices","tools":["get_invoice"],"maxSteps":3}));
        harness.fakes.tools.push(std::sync::Arc::new(DownTool));
        harness
            .fakes
            .models
            .push_tool_call("get_invoice", json!({"id":"inv1"}))
            .push_text("Billing is down; try later.");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["answer"], "Billing is down; try later.");
        assert!(out.ports["out"][0].json["activity"][0]["result"]["error"].as_str().unwrap().contains("unreachable"));
    }

    #[tokio::test]
    async fn r05_agent_uses_declared_api_operation_without_a_data_input() {
        let mut harness = NodeHarness::new(super::AiAgent).settings(json!({
            "instructions":"Check invoices",
            "apiTools":{"spec":"file_api","operations":["get_invoice"]}
        }));
        harness.fakes.tools.push(std::sync::Arc::new(InvoiceTool));
        harness.fakes.models.push_tool_call("get_invoice", json!({"id":"inv1"})).push_text("Invoice paid.");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["activity"][0]["tool"], "get_invoice");
    }

    #[tokio::test]
    async fn q05_agent_skips_missing_declared_tool_and_reports_warning() {
        let harness = NodeHarness::new(super::AiAgent).settings(json!({
            "instructions":"Answer the customer", "tools":["get_invoice"]
        }));
        harness.fakes.models.push_text("I can help with general questions.");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["answer"], "I can help with general questions.");
        assert_eq!(out.ports["out"][0].json["activity"][0]["warning"], "tool get_invoice is unavailable");
        assert!(harness.fakes.models.calls()[0].1.tools.is_empty());
    }

    #[tokio::test]
    async fn q05_agent_recovers_when_model_calls_an_unavailable_tool() {
        let harness = NodeHarness::new(super::AiAgent).settings(json!({"instructions":"Help"}));
        harness
            .fakes
            .models
            .push_tool_call("get_invoice", json!({"id":"inv1"}))
            .push_text("I cannot check that invoice.");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["answer"], "I cannot check that invoice.");
        assert_eq!(out.ports["out"][0].json["activity"][0]["warning"], "tool get_invoice is unavailable");
        assert!(harness.fakes.models.calls()[1].1.messages.iter().any(|message| {
            message.role == il_sdk::services::models::ChatRole::Tool
                && message.content.contains("get_invoice is unavailable")
        }));
    }
}
