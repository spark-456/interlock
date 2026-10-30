//! What a flow carries into a bundle (its prompts, the files it refers to, the connections its steps
//! use) and the reverse steps the engine applies when it loads one.

use std::collections::{BTreeMap, BTreeSet};

use il_core::Flow;
use il_core::manifest::CredentialKind;
use serde_json::Value;

use super::manifest::PROMPT_REF;

/// Moves each step's long-text settings (instructions, prompts, questions) to `prompts/<step>.<field>.md`
/// and leaves an `@prompts/…` reference. Returns the prompt files.
pub(super) fn take_prompts(flow: &mut Flow) -> Vec<(String, String)> {
    let mut files = Vec::new();
    for node in &mut flow.nodes {
        let Some(exec) = il_sdk::registry::node(&node.node_type) else { continue };
        let Some(props) = exec.manifest().settings_schema["properties"].as_object() else { continue };
        for key in props.iter().filter(|(_, p)| p["x-widget"] == "longtext").map(|(k, _)| k) {
            let Some(text) = node.settings.get(key).and_then(Value::as_str).filter(|t| !t.trim().is_empty()) else {
                continue;
            };
            let path = format!("prompts/{}.{}.md", safe(&node.id), safe(key));
            // One newline is added so editors leave the file alone; `resolve_prompts` removes it.
            files.push((path.clone(), format!("{text}\n")));
            node.settings[key] = Value::String(format!("@{path}"));
        }
    }
    files
}

/// Fills each `@prompts/…` setting from the bundle; `read` returns a bundle file's text.
pub fn resolve_prompts(flow: &mut Flow, read: impl Fn(&str) -> Option<String>) -> Result<(), String> {
    for node in &mut flow.nodes {
        let Some(settings) = node.settings.as_object_mut() else { continue };
        for value in settings.values_mut() {
            let Some(path) = value.as_str().filter(|v| v.starts_with(PROMPT_REF)).map(|v| v[1..].to_owned()) else {
                continue;
            };
            let text = read(&path).ok_or_else(|| format!("{path} is missing"))?;
            let text = text.strip_suffix('\n').map(|t| t.strip_suffix('\r').unwrap_or(t)).unwrap_or(&text);
            *value = Value::String(text.to_owned());
        }
    }
    Ok(())
}

/// Every stored file id (`file_<ULID>`) mentioned in the flow's step settings.
pub(super) fn file_ids(flow: &Flow) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for node in &flow.nodes {
        visit(&node.settings, &mut |text| {
            let mut rest = text;
            while let Some(at) = rest.find("file_") {
                let tail = &rest[at + 5..];
                let len = tail.chars().take_while(char::is_ascii_alphanumeric).count();
                if len == 26 {
                    ids.insert(format!("file_{}", &tail[..26]));
                }
                rest = &tail[len..];
            }
        });
    }
    ids
}

fn visit(value: &Value, f: &mut impl FnMut(&str)) {
    match value {
        Value::String(text) => f(text),
        Value::Array(items) => items.iter().for_each(|v| visit(v, f)),
        Value::Object(map) => map.values().for_each(|v| visit(v, f)),
        _ => {}
    }
}

/// Points the flow's file references at the stored copies (`old id → new id`).
pub fn rewrite_file_ids(flow: &mut Flow, ids: &BTreeMap<String, String>) {
    fn rewrite(value: &mut Value, ids: &BTreeMap<String, String>) {
        match value {
            Value::String(text) => {
                for (old, new) in ids {
                    if text.contains(old.as_str()) {
                        *text = text.replace(old.as_str(), new);
                    }
                }
            }
            Value::Array(items) => items.iter_mut().for_each(|v| rewrite(v, ids)),
            Value::Object(map) => map.values_mut().for_each(|v| rewrite(v, ids)),
            _ => {}
        }
    }
    for node in &mut flow.nodes {
        rewrite(&mut node.settings, ids);
    }
}

/// One step's use of a connection.
pub(super) struct Slot {
    pub credential_id: String,
    pub app: String,
    pub kind: CredentialKind,
    pub label: String,
    pub step: String,
}

/// The connections the flow's steps use: every bound one, and every required one that isn't bound
/// yet, which gets an id (bound in the exported flow) so the bundle can supply it from env.
pub(super) fn credential_slots(flow: &mut Flow) -> Vec<Slot> {
    let mut slots = Vec::new();
    for node in &mut flow.nodes {
        let Some(exec) = il_sdk::registry::node(&node.node_type) else { continue };
        for spec in &exec.manifest().credentials {
            let id = match node.credentials.get(&spec.key) {
                Some(id) => id.clone(),
                None if spec.optional || node.disabled => continue,
                None => {
                    let id = format!("cred_{}_{}", safe(&node.id), safe(&spec.key));
                    node.credentials.insert(spec.key.clone(), id.clone());
                    id
                }
            };
            slots.push(Slot {
                credential_id: id,
                app: spec.app.clone(),
                kind: spec.kind,
                label: spec.label.clone(),
                step: node.name.clone(),
            });
        }
    }
    slots
}

/// The secret fields a connection of this kind holds, for connections never saved in the workspace.
pub(super) fn default_fields(kind: CredentialKind) -> Vec<String> {
    let fields: &[&str] = match kind {
        CredentialKind::OAuth2 => &["refresh_token"],
        CredentialKind::ApiKey => &["api_key"],
        CredentialKind::BotToken => &["token"],
        CredentialKind::AppPassword => &["identifier", "app_password"],
        CredentialKind::Webhook => &["url"],
    };
    fields.iter().map(|f| (*f).to_owned()).collect()
}

/// `SLACK_BOT_TOKEN` style: upper case, words joined by `_`.
pub(super) fn env_name(parts: &[&str]) -> String {
    let joined = parts.join("_").to_ascii_uppercase();
    let mapped: String = joined.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    mapped.split('_').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("_")
}

/// A path segment: letters, digits, `.`, `-` and `_` only, never starting with a dot.
pub(super) fn safe(name: &str) -> String {
    let cleaned: String =
        name.chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' }).collect();
    let trimmed = cleaned.trim_start_matches('.');
    if trimmed.is_empty() { "file".into() } else { trimmed.chars().take(80).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn flow(settings: Value) -> Flow {
        serde_json::from_value(json!({"id":"flw_1","workspaceId":"ws_a","version":3,"name":"Help","kind":"app",
            "nodes":[{"id":"n_agent","type":"test.export","name":"Agent","settings":settings}],"edges":[]}))
        .unwrap()
    }

    #[test]
    fn q36_prompts_round_trip_through_files() {
        let mut f = flow(json!({"instructions":"Be kind.\nCite sources.","input":"{{ item.message }}"}));
        let files = take_prompts(&mut f);
        assert_eq!(files, vec![("prompts/n_agent.instructions.md".into(), "Be kind.\nCite sources.\n".into())]);
        assert_eq!(f.nodes[0].settings["instructions"], "@prompts/n_agent.instructions.md");
        assert_eq!(f.nodes[0].settings["input"], "{{ item.message }}", "expressions stay in the flow");
        let edited = "Be brief.\r\n".to_owned();
        resolve_prompts(&mut f, |path| (path == "prompts/n_agent.instructions.md").then(|| edited.clone())).unwrap();
        assert_eq!(f.nodes[0].settings["instructions"], "Be brief.");
        let mut missing = flow(json!({"instructions":"@prompts/gone.md"}));
        assert_eq!(resolve_prompts(&mut missing, |_| None).unwrap_err(), "prompts/gone.md is missing");
    }

    #[test]
    fn q36_file_ids_are_found_and_rewritten_everywhere() {
        let id = "file_01JABCDEFGHJKMNPQRSTVWXYZ0";
        let mut f = flow(json!({"apiTools":{"spec":id},"sampleItems":[{"files":{"a":{"id":id,
            "previewUrl":format!("/api/files/{id}")}}}],"note":"file_short"}));
        assert_eq!(file_ids(&f).into_iter().collect::<Vec<_>>(), vec![id.to_owned()]);
        rewrite_file_ids(&mut f, &BTreeMap::from([(id.to_owned(), "file_NEW".to_owned())]));
        let s = &f.nodes[0].settings;
        assert_eq!((s["apiTools"]["spec"].as_str(), s["note"].as_str()), (Some("file_NEW"), Some("file_short")));
        assert_eq!(s["sampleItems"][0]["files"]["a"]["previewUrl"], "/api/files/file_NEW");
    }

    #[test]
    fn q36_names_are_safe() {
        assert_eq!(env_name(&["slack", "bot_token"]), "SLACK_BOT_TOKEN");
        assert_eq!(env_name(&["Larkspur API", "2", "api_key"]), "LARKSPUR_API_2_API_KEY");
        assert_eq!(safe("../etc/passwd"), "_etc_passwd");
        assert_eq!(safe("Refund policy (2026).pdf"), "Refund_policy__2026_.pdf");
    }
}
