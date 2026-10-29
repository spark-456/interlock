//! Build attachments in the flow (K04, AR-1): Knowledge attachments become the sources of the flow's
//! knowledge base step, and Instructions (documents or transcribed voice memos) join every agent's
//! instructions.

use il_core::dto::{Attachment, AttachmentKind, AttachmentRole};
use il_core::{Flow, Node, Position};
use il_sdk::Services;
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

/// Puts the attachments into `flow` and returns a log line for each change.
pub(super) fn apply(flow: &mut Flow, attachments: &[Attachment], instructions: &[(String, String)]) -> Vec<String> {
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

    use super::apply;

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
        let log = apply(&mut f, &attachments, &[("memo.mp3".into(), "Keep answers short.".into())]);
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
        apply(&mut f, &attachments, &[]);
        assert_eq!(f.node("n_kb1").unwrap().settings["sources"], json!([{"kind": "file", "value": "fil_a"}]));
        assert_eq!(f.node("n_agent").unwrap().settings["knowledgeBases"], json!(["n_kb1"]));
    }
}
