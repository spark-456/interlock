//! Goal + attachments → Flow JSON with the Architect role: the model writes a compact draft, the
//! draft becomes a laid-out flow, and validation errors go back to the model (AR-3, AR-4, AR-10).

mod attach;
mod build;
mod progress;

use std::collections::BTreeMap;

use il_core::events::BuildQuestion;
use il_core::{Edge, Flow, FlowKind, FlowMeta, FlowSettings, Group, Json, Node, PortRef, Position};
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelError, ModelGateway, ModelRole};
use serde::Deserialize;
use serde_json::json;

pub use build::{Architect, BuildCaps};

use crate::catalog::Catalog;
use crate::validate::{Issue, validate};

/// Re-prompts with validation errors before giving up.
pub const MAX_REPAIRS: usize = 3;
/// At most this many clarifying questions (AR-2).
const MAX_QUESTIONS: usize = 3;

const SYSTEM: &str = include_str!("../../../../prompts/architect/system.md");

/// What the model writes: steps without positions, edges by node id with optional ports.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub name: String,
    #[serde(default)]
    pub kind: Option<FlowKind>,
    #[serde(default)]
    pub questions: Vec<BuildQuestion>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub nodes: Vec<DraftNode>,
    #[serde(default)]
    pub edges: Vec<DraftEdge>,
    /// One item the trigger would emit, for the test run (AR-6).
    #[serde(default)]
    pub sample: Option<Json>,
}

/// A step in a draft.
#[derive(Debug, Clone, Deserialize)]
pub struct DraftNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub settings: Json,
}

/// An edge in a draft; ports default to the step's first output and input.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftEdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub from_port: Option<String>,
    #[serde(default)]
    pub to_port: Option<String>,
}

/// JSON Schema of a draft, for structured output.
fn draft_schema() -> Json {
    let text = json!({"type": "string"});
    json!({
        "type": "object",
        "required": ["name", "nodes", "edges"],
        "properties": {
            "name": text,
            "kind": {"type": "string", "enum": ["automation", "app"]},
            "questions": {"type": "array", "items": {"type": "object", "required": ["id", "text", "options"],
                "properties": {"id": text, "text": text, "options": {"type": "array", "items": text}}}},
            "notes": {"type": "array", "items": text},
            "groups": {"type": "array", "items": {"type": "object", "required": ["id", "label"],
                "properties": {"id": text, "label": text}}},
            "nodes": {"type": "array", "items": {"type": "object", "required": ["id", "type", "name"],
                "properties": {"id": text, "type": text, "name": text, "group": text,
                    "settings": {"type": "object", "additionalProperties": true}}}},
            "edges": {"type": "array", "items": {"type": "object", "required": ["from", "to"],
                "properties": {"from": text, "to": text, "fromPort": text, "toPort": text}}},
            "sample": {}
        }
    })
}

/// A laid-out flow from a draft. Unknown types keep default ports; validation reports them.
pub fn assemble(draft: &Draft, catalog: &Catalog, goal: &str) -> Flow {
    let nodes = draft
        .nodes
        .iter()
        .map(|d| Node {
            id: d.id.clone(),
            node_type: d.node_type.clone(),
            type_version: catalog.get(&d.node_type).map_or(1, |m| m.version),
            name: d.name.clone(),
            position: Position { x: 0.0, y: 0.0 },
            group: d.group.clone(),
            settings: if d.settings.is_null() { json!({}) } else { d.settings.clone() },
            credentials: BTreeMap::new(),
            disabled: false,
            notes: None,
        })
        .collect();
    let first_port = |id: &str, outputs: bool| {
        let m = draft.nodes.iter().find(|n| n.id == id).and_then(|n| catalog.get(&n.node_type));
        let ports = m.map(|m| if outputs { &m.outputs } else { &m.inputs });
        ports.and_then(|p| p.first()).map_or(if outputs { "out" } else { "in" }, |p| p.name.as_str()).to_string()
    };
    let mut edges: Vec<Edge> = Vec::new();
    for e in &draft.edges {
        let short = |id: &str| id.strip_prefix("n_").unwrap_or(id).to_string();
        let base = format!("e_{}_{}", short(&e.from), short(&e.to));
        let mut id = base.clone();
        for k in 2.. {
            if !edges.iter().any(|x| x.id == id) {
                break;
            }
            id = format!("{base}_{k}");
        }
        edges.push(Edge {
            id,
            from: PortRef {
                node: e.from.clone(),
                port: e.from_port.clone().unwrap_or_else(|| first_port(&e.from, true)),
            },
            to: PortRef { node: e.to.clone(), port: e.to_port.clone().unwrap_or_else(|| first_port(&e.to, false)) },
        });
    }
    let mut flow = Flow {
        id: il_sdk::id("flw"),
        workspace_id: String::new(),
        version: 1,
        name: draft.name.clone(),
        kind: draft.kind.unwrap_or(FlowKind::Automation),
        nodes,
        edges,
        groups: draft.groups.clone(),
        settings: FlowSettings::default(),
        meta: FlowMeta { created_by: Some("architect".into()), goal: Some(goal.into()) },
    };
    drop_unknown_bases(&mut flow);
    crate::layout::apply(&mut flow);
    flow
}

/// Removes knowledge bases an agent names that are neither a knowledge step in the flow nor a saved
/// base (`kb_` + ULID). Models invent ids like `kb_help_center`, and searching those finds nothing.
fn drop_unknown_bases(flow: &mut Flow) {
    let steps: Vec<String> =
        flow.nodes.iter().filter(|n| n.node_type == "knowledge.base").map(|n| n.id.clone()).collect();
    let saved = |id: &str| {
        id.strip_prefix("kb_")
            .is_some_and(|u| u.len() == 26 && u.chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()))
    };
    for node in flow.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
        if let Some(bases) = node.settings.get_mut("knowledgeBases").and_then(|b| b.as_array_mut()) {
            bases.retain(|b| b.as_str().is_some_and(|id| steps.iter().any(|s| s == id) || saved(id)));
        }
    }
}

/// What compiling produced.
#[derive(Debug)]
pub enum Outcome {
    /// A valid flow, the model's notes for the build log, and the sample trigger item.
    Flow { flow: Box<Flow>, notes: Vec<String>, sample: Json },
    /// Clarifying questions to answer before compiling again.
    Questions(Vec<BuildQuestion>),
}

/// The result of [`compile`], with what it cost and how many problems each repair round fixed.
#[derive(Debug)]
pub struct Compiled {
    pub outcome: Outcome,
    pub cost_usd: f64,
    pub repairs: Vec<usize>,
}

/// Compile errors.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("model call failed: {0}")]
    Model(#[from] ModelError),
    #[error("the flow still had problems after {MAX_REPAIRS} repairs: {}", list(.0))]
    Invalid(Vec<Issue>),
}

fn list(issues: &[Issue]) -> String {
    issues.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
}

/// Asks the Architect model for a flow, repairing it with validation errors up to [`MAX_REPAIRS`] times.
/// `brief` is the goal with attachments and answers; questions are returned only when `may_ask`.
pub async fn compile(
    models: &dyn ModelGateway,
    catalog: &Catalog,
    goal: &str,
    brief: &str,
    may_ask: bool,
) -> Result<Compiled, CompileError> {
    draft_loop(models, catalog, goal, vec![system(catalog), ChatMessage::user(brief)], may_ask).await
}

fn system(catalog: &Catalog) -> ChatMessage {
    ChatMessage::system(format!("{SYSTEM}\nCatalog:\n{}", catalog.prompt()))
}

/// A revised flow and what revising it cost.
#[derive(Debug)]
pub struct Revised {
    pub flow: Flow,
    pub notes: Vec<String>,
    pub cost_usd: f64,
}

/// Asks for `flow` changed as `request` says. The result keeps the flow's id, and steps whose id
/// and type survive keep their position, credentials and disabled state.
pub async fn revise(
    models: &dyn ModelGateway,
    catalog: &Catalog,
    flow: &Flow,
    request: &str,
) -> Result<Revised, CompileError> {
    let current = serde_json::to_string_pretty(&to_draft(flow)).unwrap_or_default();
    let user = format!(
        "This is the current flow:\n{current}\n\n{request}\n\nChange only what that needs, keep the ids of steps \
         you keep, and reply with the whole revised JSON object."
    );
    let goal = flow.meta.goal.clone().unwrap_or_default();
    let compiled = draft_loop(models, catalog, &goal, vec![system(catalog), ChatMessage::user(user)], false).await?;
    let Outcome::Flow { flow: next, notes, .. } = compiled.outcome else {
        return Err(CompileError::Invalid(vec![Issue { path: "reply".into(), message: "asked questions".into() }]));
    };
    let mut next = *next;
    for node in &mut next.nodes {
        if let Some(old) = flow.node(&node.id).filter(|old| old.node_type == node.node_type) {
            node.position = old.position;
            node.credentials = old.credentials.clone();
            node.disabled = old.disabled;
            node.notes = old.notes.clone();
        }
    }
    next.id = flow.id.clone();
    next.workspace_id = flow.workspace_id.clone();
    next.version = flow.version;
    next.settings = flow.settings.clone();
    next.meta = flow.meta.clone();
    Ok(Revised { flow: next, notes, cost_usd: compiled.cost_usd })
}

/// A flow in the draft format the model writes, for revising it.
pub fn to_draft(flow: &Flow) -> Json {
    json!({
        "name": flow.name,
        "kind": flow.kind,
        "groups": flow.groups,
        "nodes": flow.nodes.iter().map(|n| json!({
            "id": n.id, "type": n.node_type, "name": n.name, "group": n.group, "settings": n.settings,
        })).collect::<Vec<_>>(),
        "edges": flow.edges.iter().map(|e| json!({
            "from": e.from.node, "fromPort": e.from.port, "to": e.to.node, "toPort": e.to.port,
        })).collect::<Vec<_>>(),
    })
}

async fn draft_loop(
    models: &dyn ModelGateway,
    catalog: &Catalog,
    goal: &str,
    mut messages: Vec<ChatMessage>,
    may_ask: bool,
) -> Result<Compiled, CompileError> {
    let mut cost_usd = 0.0;
    let mut repairs = Vec::new();
    loop {
        let req = ChatRequest {
            messages: messages.clone(),
            json_schema: Some(draft_schema()),
            temperature: Some(0.2),
            ..Default::default()
        };
        let resp = models.chat(ModelRole::Architect, req).await?;
        cost_usd += resp.usage.cost_usd;
        let issues = match parse(&resp.content) {
            Ok(draft) if may_ask && !draft.questions.is_empty() => {
                let questions = draft.questions.into_iter().take(MAX_QUESTIONS).collect();
                return Ok(Compiled { outcome: Outcome::Questions(questions), cost_usd, repairs });
            }
            Ok(draft) => {
                let flow = assemble(&draft, catalog, goal);
                let issues = validate(&flow, catalog);
                if issues.is_empty() {
                    return Ok(Compiled {
                        outcome: Outcome::Flow {
                            flow: Box::new(flow),
                            notes: draft.notes,
                            sample: draft.sample.unwrap_or_else(|| json!({})),
                        },
                        cost_usd,
                        repairs,
                    });
                }
                issues
            }
            Err(e) => vec![Issue { path: "reply".into(), message: format!("is not a valid flow draft: {e}") }],
        };
        if repairs.len() == MAX_REPAIRS {
            return Err(CompileError::Invalid(issues));
        }
        repairs.push(issues.len());
        messages.push(ChatMessage::assistant(resp.content));
        messages.push(ChatMessage::user(format!(
            "The flow has these problems. Fix them and reply with the whole corrected JSON object. Keep every \
             setting that was right, and fill each missing one with the keys the catalog lists for that step type.\n- {}",
            issues.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n- ")
        )));
    }
}

/// Parses a draft, tolerating text or code fences around the JSON object.
fn parse(content: &str) -> Result<Draft, serde_json::Error> {
    let body = match (content.find('{'), content.rfind('}')) {
        (Some(a), Some(b)) if a < b => &content[a..=b],
        _ => content,
    };
    serde_json::from_str(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q11_agents_keep_only_real_knowledge_bases() {
        let draft: Draft = serde_json::from_value(json!({
            "name": "Help", "edges": [],
            "nodes": [
                {"id": "n_kb", "type": "knowledge.base", "name": "Docs", "settings": {}},
                {"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {
                    "knowledgeBases": ["kb_help_center", "n_kb", "kb_01M64RCE581WKXW9XSECGGBPFK", "n_missing"]}}
            ]
        }))
        .unwrap();
        let flow = assemble(&draft, &Catalog::default(), "help");
        assert_eq!(
            flow.node("n_agent").unwrap().settings["knowledgeBases"],
            json!(["n_kb", "kb_01M64RCE581WKXW9XSECGGBPFK"])
        );
    }

    #[test]
    fn q01_schema_leaves_settings_open_and_any_sample() {
        let schema = draft_schema();
        assert_eq!(schema["properties"]["nodes"]["items"]["properties"]["settings"]["additionalProperties"], true);
        let draft = json!({
            "name": "Triage", "edges": [],
            "nodes": [{"id": "n_slack", "type": "app.slack.send", "name": "Post", "settings": {"channel": "#support"}}],
            "sample": [{"subject": "Refund?"}]
        });
        assert!(il_sdk::validate::validate_json(&schema, &draft).is_ok());
    }
}
