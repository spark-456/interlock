//! Building flows from goals, the step catalog, and knowledge bases.

use il_core::dto::{Attachment, AttachmentKind, AttachmentRole, BuildAnswers, BuildRequest};
use il_sdk::registry::RouteCtx;
use il_sdk::services::knowledge::SourceInput;
use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::{clip, string_arg, strings_arg};
use crate::mcp::tool;

/// Log lines `get_build` returns: the latest ones say where the build is.
const RECENT_LOG: usize = 12;

pub(super) fn tools() -> Vec<Tool> {
    let build = json!({"type": "string", "description": "Build id from create_flow_from_goal"});
    vec![
        tool(
            "create_flow_from_goal",
            &format!(
                "Start building a flow from a goal in plain words, optionally with links to learn from (help \
                 centers, docs), OpenAPI specs it may call, and local files (documents, voice memos). Upload each \
                 file first with `{}`; it returns the file_id. Takes one to a few minutes: follow it with get_build.",
                crate::mcp::upload::COMMAND
            ),
            json!({"type": "object", "properties": {
                "goal": {"type": "string"},
                "files": {"type": "array", "description": "Uploaded files", "items": {"type": "object", "properties": {
                    "file_id": {"type": "string"},
                    "role": {"type": "string", "enum": ["knowledge", "instructions"], "description":
                        "knowledge: documents to answer from (default); instructions: rules for the agents (default for audio)"}
                }, "required": ["file_id"]}},
                "links": {"type": "array", "items": {"type": "string"}, "description": "Websites it should know"},
                "api_specs": {"type": "array", "items": {"type": "string"},
                    "description": "OpenAPI spec URLs; their safe operations become the agents' tools"}
            }, "required": ["goal"]}),
        ),
        tool(
            "get_build",
            "Where a build is: status (building, waiting_for_answers, ready, failed), recent log, test results, \
             questions to answer, and the flow id once ready.",
            json!({"type": "object", "properties": {"build_id": build}, "required": ["build_id"]}),
        ),
        tool(
            "answer_build_questions",
            "Answer the questions a build asked (question id → answer), so it continues.",
            json!({"type": "object", "properties": {"build_id": build,
                "answers": {"type": "object", "additionalProperties": {"type": "string"}}},
                "required": ["build_id", "answers"]}),
        ),
        tool(
            "list_step_types",
            "The step types flows are made of (triggers, AI, logic, apps, output) with their settings, for \
             editing a flow with update_flow. Filter with a word like \"slack\" or \"agent\".",
            json!({"type": "object", "properties": {"query": {"type": "string"}}}),
        ),
        tool(
            "list_knowledge_bases",
            "The workspace's knowledge bases (documents and websites agents search).",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "add_documents",
            "Add text documents to a knowledge base.",
            json!({"type": "object", "properties": {"kb_id": {"type": "string"}, "documents": {"type": "array",
                "items": {"type": "object", "properties": {"title": {"type": "string"}, "text": {"type": "string"}},
                "required": ["title", "text"]}}}, "required": ["kb_id", "documents"]}),
        ),
    ]
}

pub(super) async fn call(
    ctx: &RouteCtx,
    workspace_id: &str,
    name: &str,
    arguments: &JsonObject,
) -> Option<Result<Value, String>> {
    Some(match name {
        "create_flow_from_goal" => create(ctx, workspace_id, arguments).await,
        "get_build" => get(ctx, workspace_id, arguments).await,
        "answer_build_questions" => answer(ctx, workspace_id, arguments).await,
        "list_step_types" => Ok(step_types(arguments.get("query").and_then(Value::as_str).unwrap_or_default())),
        "list_knowledge_bases" => {
            ctx.services.knowledge.list_bases(workspace_id).await.map(Value::from).map_err(|e| e.to_string())
        }
        "add_documents" => add_documents(ctx, workspace_id, arguments).await,
        _ => return None,
    })
}

/// Where the server records which workspace started a build (shared with `/api/builds`).
fn owner_key(build_id: &str) -> String {
    format!("ar:build:workspace:{build_id}")
}

async fn create(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let goal = string_arg(arguments, "goal")?;
    let linked = |key, kind| {
        strings_arg(arguments, key).into_iter().map(move |url| Attachment {
            kind,
            name: url.clone(),
            file_id: None,
            url: Some(url),
            role: AttachmentRole::Knowledge,
        })
    };
    let mut attachments: Vec<Attachment> =
        linked("links", AttachmentKind::Link).chain(linked("api_specs", AttachmentKind::Api)).collect();
    for entry in arguments.get("files").and_then(Value::as_array).into_iter().flatten() {
        attachments.push(uploaded(ctx, workspace_id, entry).await?);
    }
    let req = BuildRequest { goal, kind: None, attachments, hints: vec![] };
    let id = ctx.architect.start_build(workspace_id, req).await.map_err(|e| e.to_string())?;
    ctx.services.store.kv_put(&owner_key(&id), &json!(workspace_id)).await.map_err(|e| e.to_string())?;
    Ok(json!({"build_id": id, "next": "Call get_build every 20 seconds or so until status is ready or failed."}))
}

/// An uploaded file as an attachment; another workspace's file reads as missing.
async fn uploaded(ctx: &RouteCtx, workspace_id: &str, entry: &Value) -> Result<Attachment, String> {
    let id = entry["file_id"].as_str().unwrap_or_default();
    let missing = || format!("file {id} not found; upload it to /mcp/files first");
    let owner = ctx.services.files.owner(id).await.map_err(|_| missing())?;
    if owner.as_deref() != Some(workspace_id) {
        return Err(missing());
    }
    let (file, _) = ctx.services.files.get(id).await.map_err(|_| missing())?;
    let audio = file.mime.starts_with("audio/");
    // As in the app, a voice memo is the person's instructions unless they say otherwise.
    let role = match entry["role"].as_str() {
        Some("instructions") => AttachmentRole::Instructions,
        Some(_) => AttachmentRole::Knowledge,
        None if audio => AttachmentRole::Instructions,
        None => AttachmentRole::Knowledge,
    };
    let kind = if audio { AttachmentKind::Audio } else { AttachmentKind::File };
    Ok(Attachment { kind, name: file.name, file_id: Some(file.id), url: None, role })
}

async fn owned(ctx: &RouteCtx, workspace_id: &str, build_id: &str) -> Result<(), String> {
    let owner = ctx.services.store.kv_get(&owner_key(build_id)).await.map_err(|e| e.to_string())?;
    if owner.as_ref().and_then(Value::as_str) == Some(workspace_id) { Ok(()) } else { Err("build not found".into()) }
}

async fn get(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let id = string_arg(arguments, "build_id")?;
    owned(ctx, workspace_id, &id).await?;
    let events = ctx.services.store.build_events(&id, 0).await.map_err(|e| e.to_string())?;
    let mut out = json!({"build_id": id, "status": "building", "stage": null, "log": [], "cost_usd": 0.0});
    let mut log = Vec::new();
    let mut questions = None;
    for event in events.iter().map(|e| json!(e)) {
        let data = &event["data"];
        match event["type"].as_str().unwrap_or_default() {
            "build.stage" => out["stage"] = data["stage"].clone(),
            "build.log" => log.push(clip(data["text"].as_str().unwrap_or_default(), 300)),
            "build.cost" => out["cost_usd"] = data["costUsd"].clone(),
            "build.test_result" => out["tests"] = json!({"passed": data["passed"], "failed": data["failed"]}),
            "build.question" => {
                questions = Some(data["questions"].clone());
                continue;
            }
            "build.ready" => {
                out["status"] = json!("ready");
                out["flow_id"] = data["flowId"].clone();
                out["next"] = json!("Read it with get_flow, try it with test_flow, turn it on with set_flow_active.");
            }
            "build.failed" => {
                out["status"] = json!("failed");
                out["error"] = data["error"].clone();
            }
            _ => {}
        }
        // Anything after the questions means they were answered.
        questions = None;
    }
    if let Some(questions) = questions {
        out["status"] = json!("waiting_for_answers");
        out["questions"] = questions;
        out["next"] = json!("Answer with answer_build_questions (question id → one of its options or your own).");
    }
    out["log"] = json!(log[log.len().saturating_sub(RECENT_LOG)..]);
    Ok(out)
}

async fn answer(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let id = string_arg(arguments, "build_id")?;
    owned(ctx, workspace_id, &id).await?;
    let answers = arguments
        .get("answers")
        .and_then(Value::as_object)
        .ok_or("answers must be an object of question id → answer")?
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().map_or_else(|| v.to_string(), str::to_owned)))
        .collect();
    ctx.architect.answer(&id, BuildAnswers { answers }).await.map_err(|e| e.to_string())?;
    Ok(json!({"build_id": id, "answered": true}))
}

fn step_types(query: &str) -> Value {
    let q = query.to_lowercase();
    let types: Vec<Value> = il_sdk::registry::catalog()
        .into_iter()
        .filter(|m| {
            q.is_empty() || [&m.type_id, &m.title, &m.description].iter().any(|s| s.to_lowercase().contains(&q))
        })
        .map(|m| {
            json!({"type": m.type_id, "title": m.title, "description": m.description,
                "inputs": m.inputs.iter().map(|p| &p.name).collect::<Vec<_>>(),
                "outputs": m.outputs.iter().map(|p| &p.name).collect::<Vec<_>>(),
                "settings": m.settings_schema})
        })
        .collect();
    json!(types)
}

async fn add_documents(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let kb_id = string_arg(arguments, "kb_id")?;
    if !ctx.services.knowledge.owns_base(workspace_id, &kb_id).await.map_err(|e| e.to_string())? {
        return Err("knowledge base not found".into());
    }
    let documents = arguments.get("documents").and_then(Value::as_array).ok_or("documents must be an array")?;
    let sources: Vec<_> = documents
        .iter()
        .map(|value| -> Result<SourceInput, &str> {
            let field = |key| value.get(key).and_then(Value::as_str).filter(|text| !text.is_empty());
            let title = field("title").ok_or("document title is required")?;
            let text = field("text").ok_or("document text is required")?;
            Ok(SourceInput::Text { title: title.into(), text: text.into() })
        })
        .collect::<Result<_, _>>()?;
    let ids = ctx.services.knowledge.add_sources(&kb_id, sources).await.map_err(|e| e.to_string())?;
    Ok(json!({"source_ids": ids}))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use il_core::Flow;
    use il_core::dto::{AskDiff, AttachmentKind, AttachmentRole, BuildAnswers, BuildRequest};
    use il_sdk::registry::RouteCtx;
    use il_sdk::services::architect::ArchitectError;
    use il_sdk::services::{ArchitectService, NoActivation, NoFlowTools};
    use il_sdk::testkit::{FakeRunner, FakeServices};
    use serde_json::json;

    #[derive(Default)]
    struct Recording(Mutex<Vec<BuildRequest>>);

    #[async_trait]
    impl ArchitectService for Recording {
        async fn start_build(&self, _ws: &str, req: BuildRequest) -> Result<String, ArchitectError> {
            self.0.lock().unwrap().push(req);
            Ok("bld_1".into())
        }
        async fn answer(&self, _id: &str, _answers: BuildAnswers) -> Result<(), ArchitectError> {
            Ok(())
        }
        async fn ask(&self, _flow: &Flow, _message: &str, _focus: &[String]) -> Result<AskDiff, ArchitectError> {
            Err(ArchitectError::Failed("unused".into()))
        }
    }

    #[tokio::test]
    async fn q25_builds_take_links_and_api_specs() {
        let architect = Arc::new(Recording::default());
        let ctx = RouteCtx {
            services: FakeServices::default().services(),
            runner: Arc::new(FakeRunner::default()),
            architect: architect.clone(),
            flow_tools: Arc::new(NoFlowTools),
            activation: Arc::new(NoActivation),
            config: json!({}),
        };
        let args = json!({"goal": "support agent", "links": ["https://help.example"],
            "api_specs": ["https://api.example/openapi.json"]});
        super::create(&ctx, "ws_a", args.as_object().unwrap()).await.unwrap();
        let req = architect.0.lock().unwrap().remove(0);
        let kinds: Vec<_> = req.attachments.iter().map(|a| (a.kind, a.url.clone().unwrap())).collect();
        assert_eq!(
            kinds,
            [
                (AttachmentKind::Link, "https://help.example".to_string()),
                (AttachmentKind::Api, "https://api.example/openapi.json".to_string())
            ]
        );
    }

    #[tokio::test]
    async fn q31_builds_take_uploaded_files_of_their_own_workspace() {
        let architect = Arc::new(Recording::default());
        let fakes = FakeServices::default();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: architect.clone(),
            flow_tools: Arc::new(NoFlowTools),
            activation: Arc::new(NoActivation),
            config: json!({}),
        };
        let files = &ctx.services.files;
        let pdf = files.put_for_workspace("ws_a", "faq.pdf", "application/pdf", "x".into()).await.unwrap();
        let memo = files.put_for_workspace("ws_a", "rules.mp3", "audio/mpeg", "x".into()).await.unwrap();
        let other = files.put_for_workspace("ws_b", "theirs.pdf", "application/pdf", "x".into()).await.unwrap();
        let args = json!({"goal": "support agent", "files": [{"file_id": pdf.id}, {"file_id": memo.id}]});
        super::create(&ctx, "ws_a", args.as_object().unwrap()).await.unwrap();
        let req = architect.0.lock().unwrap().remove(0);
        let got: Vec<_> = req.attachments.iter().map(|a| (a.kind, a.role, a.name.as_str())).collect();
        assert_eq!(
            got,
            [
                (AttachmentKind::File, AttachmentRole::Knowledge, "faq.pdf"),
                (AttachmentKind::Audio, AttachmentRole::Instructions, "rules.mp3")
            ]
        );
        let theirs = json!({"goal": "x", "files": [{"file_id": other.id}]});
        let error = super::create(&ctx, "ws_a", theirs.as_object().unwrap()).await.unwrap_err();
        assert!(error.contains("not found"), "{error}");
    }
}
