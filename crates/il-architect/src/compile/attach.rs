//! Build attachments in the flow (K04, AR-1): Knowledge attachments become the sources of the flow's
//! knowledge base step, Instructions (documents or transcribed voice memos) join every agent's
//! instructions, and an API spec's safe operations become every agent's API tools.

use il_core::dto::{Attachment, AttachmentKind, AttachmentRole};
use il_core::{Flow, Node, Position};
use il_sdk::Services;
use il_sdk::openapi::ApiOperation;
use serde_json::{Value, json};

/// Characters of one instructions attachment kept in the brief and in agent instructions.
const MAX_INSTRUCTIONS: usize = 4000;

/// The text of each Instructions attachment, by name: documents are parsed, voice memos transcribed.
pub(super) async fn instructions(services: &Services, attachments: &[Attachment]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for a in attachments.iter().filter(|a| a.role == AttachmentRole::Instructions) {
        let Some(file_id) = a.file_id.as_deref() else { continue };
        let text = if a.kind == AttachmentKind::Audio {
            match services.files.get(file_id).await {
                Ok((file, bytes)) => services.models.transcribe(&file, &bytes).await.map(|t| t.text).ok(),
                Err(_) => None,
            }
        } else {
            let parsed = services.knowledge.parse(file_id, None).await.ok();
            parsed.and_then(|p| p["text"].as_str().map(str::to_string))
        };
        match text.filter(|t| !t.trim().is_empty()) {
            Some(text) => out.push((a.name.clone(), text.chars().take(MAX_INSTRUCTIONS).collect())),
            None => tracing::warn!(attachment = %a.name, "could not read an instructions attachment"),
        }
    }
    out
}

/// An attached API spec: the file id or URL agents load it from, and its operations.
pub(super) struct ApiSpec {
    pub name: String,
    pub reference: String,
    pub operations: Vec<ApiOperation>,
}

impl ApiSpec {
    /// The operations an agent may call without the person allowing risky ones.
    pub fn safe(&self) -> Vec<&str> {
        self.operations.iter().filter(|o| !o.risky).map(|o| o.id.as_str()).collect()
    }
}

/// Reads each API attachment (uploaded file or link). A spec that can't be read is logged and skipped.
pub(super) async fn api_specs(services: &Services, attachments: &[Attachment]) -> Vec<ApiSpec> {
    let mut out = Vec::new();
    for a in attachments.iter().filter(|a| a.kind == AttachmentKind::Api) {
        let (reference, text) = match (&a.file_id, &a.url) {
            (Some(id), _) => {
                (id.clone(), services.files.get(id).await.ok().map(|(_, b)| String::from_utf8_lossy(&b).into_owned()))
            }
            (None, Some(url)) => (url.clone(), fetch(services, url).await),
            _ => continue,
        };
        match text.ok_or_else(|| "could not be read".to_string()).and_then(|t| il_sdk::openapi::operations(&t)) {
            Ok(operations) => out.push(ApiSpec { name: a.name.clone(), reference, operations }),
            Err(e) => tracing::warn!(attachment = %a.name, error = %e, "could not read an API spec attachment"),
        }
    }
    out
}

/// The first link attachment whose site can't be reached (no such host, refused, timed out), as a
/// message for the person. HTTP errors don't count: the crawler may still read a page a plain GET can't.
/// Recorded `.example` sites are never fetched.
pub(super) async fn unreachable_link(services: &Services, attachments: &[Attachment]) -> Option<String> {
    for a in attachments.iter().filter(|a| a.kind == AttachmentKind::Link) {
        let Some(url) = a.url.as_deref() else { continue };
        let host = url.split("://").nth(1).and_then(|rest| rest.split(['/', ':', '?']).next()).unwrap_or_default();
        if host == "example" || host.ends_with(".example") {
            continue;
        }
        let reached = match services.http.get(url) {
            Ok(request) => request.timeout(std::time::Duration::from_secs(15)).send().await.map(|_| ()),
            Err(e) => return Some(format!("Couldn't open {url}: {e}. Check the link and try again.")),
        };
        if let Err(e) = reached {
            tracing::info!(url, error = %e, "link attachment unreachable");
            return Some(format!("Couldn't open {url}: the site can't be reached. Check the link and try again."));
        }
    }
    None
}

async fn fetch(services: &Services, url: &str) -> Option<String> {
    services.http.get(url).ok()?.send().await.ok()?.error_for_status().ok()?.text().await.ok()
}

/// Removes API tools a fix gave an agent from a spec the flow didn't use before the fix: models
/// invent spec URLs (`https://api.<site>/openapi.json`) that 404 on every run.
pub(super) fn keep_known_specs(next: &mut Flow, before: &Flow) {
    let spec = |n: &Node| n.settings["apiTools"]["spec"].as_str().map(str::to_owned);
    let known: Vec<String> = before.nodes.iter().filter_map(spec).collect();
    for node in next.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
        if spec(node).is_some_and(|s| !known.contains(&s))
            && let Some(settings) = node.settings.as_object_mut()
        {
            settings.remove("apiTools");
        }
    }
}

/// Puts the attachments into `flow` and returns a log line for each change.
pub(super) fn apply(
    flow: &mut Flow,
    attachments: &[Attachment],
    instructions: &[(String, String)],
    apis: &[ApiSpec],
    hints: &[String],
) -> Vec<String> {
    let mut log = Vec::new();
    let sources: Vec<Value> = attachments
        .iter()
        .filter(|a| a.role == AttachmentRole::Knowledge)
        .filter_map(|a| match (a.kind, &a.file_id, &a.url) {
            (AttachmentKind::Link, _, Some(url)) => Some(json!({ "kind": "url", "value": url })),
            (AttachmentKind::File | AttachmentKind::Audio, Some(id), _) => Some(json!({ "kind": "file", "value": id })),
            _ => None,
        })
        .collect();
    if !sources.is_empty() {
        let i = knowledge_step(flow);
        let node = &mut flow.nodes[i];
        let list = node.settings["sources"].as_array().cloned().unwrap_or_default();
        let mut merged = list.clone();
        merged.extend(sources.into_iter().filter(|s| !list.iter().any(|l| l["value"] == s["value"])));
        log.push(format!(
            "Added {} attachment{} to “{}”.",
            merged.len() - list.len(),
            plural(merged.len() - list.len()),
            node.name
        ));
        node.settings["sources"] = Value::Array(merged);
    }
    if !instructions.is_empty() {
        let extra: String = instructions.iter().map(|(name, text)| format!("\n\nFrom {name}:\n{text}")).collect();
        let mut agents = 0;
        for node in flow.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
            let base = node.settings["instructions"].as_str().unwrap_or_default().to_string();
            node.settings["instructions"] = json!(format!("{base}{extra}"));
            agents += 1;
        }
        if agents > 0 {
            log.push(format!("Gave {agents} assistant{} your instructions.", plural(agents)));
        }
    }
    if apis.is_empty() {
        // Without an attached spec, API tools the model wrote point at a spec nobody gave it.
        for node in flow.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
            let spec = node.settings["apiTools"]["spec"].as_str().unwrap_or_default();
            if !spec.is_empty() && !hints.iter().any(|h| h == spec) {
                if let Some(settings) = node.settings.as_object_mut() {
                    settings.remove("apiTools");
                }
                log.push(format!("Removed API actions from \u{201c}{}\u{201d}: no API spec was attached.", node.name));
            }
        }
    }
    if let Some(api) = apis.first() {
        let safe = api.safe();
        let mut agents = 0;
        for node in flow.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
            let tools = &node.settings["apiTools"];
            let chosen: Vec<&str> = match tools["spec"].as_str() {
                // Keep the model's choice of operations from this spec, minus unknown or risky ones.
                Some(spec) if spec == api.reference => tools["operations"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .filter(|id| safe.contains(id))
                    .collect(),
                // Another spec (a template's, or an id the model made up): the attached one wins.
                _ => vec![],
            };
            let operations = if chosen.is_empty() { safe.clone() } else { chosen };
            node.settings["apiTools"] = json!({ "spec": api.reference, "operations": operations });
            agents += 1;
        }
        if agents > 0 {
            let skipped = api.operations.len() - safe.len();
            let note = if skipped > 0 { format!(" ({skipped} risky left out)") } else { String::new() };
            log.push(format!(
                "Gave {agents} assistant{} {} API action{} from {}{note}.",
                plural(agents),
                safe.len(),
                plural(safe.len()),
                api.name
            ));
        }
    }
    log
}

/// The index of the flow's knowledge base step; one is added (and given to every agent) when the draft has none.
fn knowledge_step(flow: &mut Flow) -> usize {
    if let Some(i) = flow.nodes.iter().position(|n| n.node_type == "knowledge.base") {
        return i;
    }
    let id = (1..).map(|i| format!("n_kb{i}")).find(|id| flow.node(id).is_none()).unwrap_or_default();
    let x = flow.nodes.iter().map(|n| n.position.x).fold(0.0, f64::min);
    let y = flow.nodes.iter().map(|n| n.position.y).fold(0.0, f64::max) + 160.0;
    flow.nodes.push(Node {
        id: id.clone(),
        node_type: "knowledge.base".into(),
        type_version: 1,
        name: "Knowledge base".into(),
        position: Position { x, y },
        group: None,
        settings: json!({ "name": format!("{} knowledge", flow.name), "sources": [] }),
        credentials: Default::default(),
        disabled: false,
        notes: None,
    });
    for agent in flow.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
        let bases = agent.settings["knowledgeBases"].as_array().cloned().unwrap_or_default();
        agent.settings["knowledgeBases"] = Value::Array(bases.into_iter().chain([json!(id)]).collect());
    }
    flow.nodes.len() - 1
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use il_core::Flow;
    use il_core::dto::{Attachment, AttachmentKind, AttachmentRole};
    use serde_json::json;

    use super::{ApiSpec, apply};

    fn attachment(kind: AttachmentKind, role: AttachmentRole, file: Option<&str>, url: Option<&str>) -> Attachment {
        Attachment { kind, name: "a".into(), file_id: file.map(Into::into), url: url.map(Into::into), role }
    }

    fn flow(nodes: serde_json::Value) -> Flow {
        serde_json::from_value(json!({
            "id": "flw_1", "workspaceId": "ws", "name": "Help", "version": 1, "kind": "app", "nodes": nodes, "edges": []
        }))
        .unwrap()
    }

    #[test]
    fn k04_knowledge_goes_to_the_knowledge_step_and_instructions_to_agents() {
        let mut f = flow(json!([
            {"id": "n_kb", "type": "knowledge.base", "name": "Help center", "settings": {"sources": [{"kind": "url", "value": "https://help.example"}]}},
            {"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"instructions": "Be kind."}}
        ]));
        let attachments = [
            attachment(AttachmentKind::File, AttachmentRole::Knowledge, Some("fil_1"), None),
            attachment(AttachmentKind::Link, AttachmentRole::Knowledge, None, Some("https://help.example")),
        ];
        let log = apply(&mut f, &attachments, &[("memo.mp3".into(), "Keep answers short.".into())], &[], &[]);
        let sources = &f.node("n_kb").unwrap().settings["sources"];
        assert_eq!(sources.as_array().unwrap().len(), 2, "the duplicate link is kept once");
        assert_eq!(sources[1], json!({"kind": "file", "value": "fil_1"}));
        assert_eq!(
            f.node("n_agent").unwrap().settings["instructions"],
            "Be kind.\n\nFrom memo.mp3:\nKeep answers short."
        );
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn k04_a_knowledge_step_is_added_when_the_draft_has_none() {
        let mut f = flow(json!([{"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {}}]));
        let attachments = [attachment(AttachmentKind::Audio, AttachmentRole::Knowledge, Some("fil_a"), None)];
        apply(&mut f, &attachments, &[], &[], &[]);
        assert_eq!(f.node("n_kb1").unwrap().settings["sources"], json!([{"kind": "file", "value": "fil_a"}]));
        assert_eq!(f.node("n_agent").unwrap().settings["knowledgeBases"], json!(["n_kb1"]));
    }

    #[test]
    fn q11_agents_get_the_specs_safe_operations() {
        let spec = r#"{"openapi": "3.0.3", "servers": [{"url": "https://api.example.com"}], "paths": {
            "/users": {"get": {"operationId": "lookup_customer"}},
            "/users/{id}": {"delete": {"operationId": "delete_customer"}},
            "/tickets": {"post": {"operationId": "open_ticket"}}}}"#;
        let api = ApiSpec {
            name: "api.json".into(),
            reference: "fil_spec".into(),
            operations: il_sdk::openapi::operations(spec).unwrap(),
        };
        let mut f = flow(json!([
            {"id": "n_a", "type": "ai.agent", "name": "Support", "settings": {}},
            {"id": "n_b", "type": "ai.agent", "name": "Billing",
             "settings": {"apiTools": {"spec": "fil_spec", "operations": ["lookup_customer", "delete_customer", "made_up"]}}},
            {"id": "n_c", "type": "ai.agent", "name": "Old", "settings": {"apiTools": {"spec": "fil_template", "operations": ["x"]}}}
        ]));
        let log = apply(&mut f, &[], &[], &[api], &[]);
        assert_eq!(
            f.node("n_a").unwrap().settings["apiTools"],
            json!({"spec": "fil_spec", "operations": ["lookup_customer", "open_ticket"]})
        );
        assert_eq!(
            f.node("n_b").unwrap().settings["apiTools"],
            json!({"spec": "fil_spec", "operations": ["lookup_customer"]}),
            "the model's pick is kept, minus risky and unknown operations"
        );
        assert_eq!(
            f.node("n_c").unwrap().settings["apiTools"],
            json!({"spec": "fil_spec", "operations": ["lookup_customer", "open_ticket"]}),
            "a template's spec is replaced by the attached one (Q14)"
        );
        assert_eq!(log, ["Gave 3 assistants 2 API actions from api.json (1 risky left out)."]);
    }

    #[test]
    fn q16_api_tools_without_an_attached_spec_are_removed() {
        let mut f = flow(json!([
            {"id": "n_a", "type": "ai.agent", "name": "Helper",
             "settings": {"apiTools": {"spec": "https://api.made-up.example/openapi.json", "operations": ["x"]}}},
            {"id": "n_b", "type": "ai.agent", "name": "Named",
             "settings": {"apiTools": {"spec": "https://api.real.com/openapi.json", "operations": ["y"]}}}
        ]));
        let log = apply(&mut f, &[], &[], &[], &["https://api.real.com/openapi.json".into()]);
        assert!(f.node("n_a").unwrap().settings.get("apiTools").is_none());
        assert!(f.node("n_b").unwrap().settings.get("apiTools").is_some(), "a spec the person named stays");
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn q16_a_fix_cannot_add_a_new_api_spec() {
        let before = flow(json!([{"id": "n_a", "type": "ai.agent", "name": "A", "settings": {}}]));
        let mut next = flow(json!([{"id": "n_a", "type": "ai.agent", "name": "A",
            "settings": {"apiTools": {"spec": "https://api.site.com/openapi.json", "operations": ["x"]}}}]));
        super::keep_known_specs(&mut next, &before);
        assert!(next.node("n_a").unwrap().settings.get("apiTools").is_none());
    }
}
