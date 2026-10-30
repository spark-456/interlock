//! Shared message and item handling for AI nodes.

use il_sdk::{
    ctx::NodeCtx,
    node::{NodeError, NodeInput, NodeOutput},
    services::models::{ChatMessage, ChatRequest, ChatResponse, ModelRole},
};
use serde_json::{Value, json};

pub(super) fn role(settings: &Value, default: ModelRole) -> Result<ModelRole, NodeError> {
    match settings["model"].as_str() {
        None => Ok(default),
        Some("architect") => Ok(ModelRole::Architect),
        Some("agent") => Ok(ModelRole::Agent),
        Some("fast") => Ok(ModelRole::Fast),
        _ => Err(NodeError::Failed("unknown model role".into())),
    }
}

pub(super) fn text(value: &Value) -> String {
    value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string())
}

pub(super) fn input_text(settings: &Value, input: &NodeInput) -> String {
    settings
        .get("input")
        .map(text)
        .unwrap_or_else(|| input.main().first().map(|item| text(&item.json)).unwrap_or_default())
}

pub(super) async fn complete(
    ctx: &NodeCtx,
    default_role: ModelRole,
    system: String,
    user: String,
    schema: Option<Value>,
) -> Result<ChatResponse, NodeError> {
    let req = ChatRequest {
        messages: vec![ChatMessage::system(system), ChatMessage::user(user)],
        json_schema: schema,
        ..Default::default()
    };
    ctx.models().chat(role(&ctx.settings, default_role)?, req).await.map_err(|e| NodeError::Failed(e.to_string()))
}

pub(super) fn output(input: &NodeInput, field: &str, value: Value) -> NodeOutput {
    let mut item = input.main().first().cloned().unwrap_or_default();
    if !item.json.is_object() {
        item.json = json!({});
    }
    item.json[field] = value;
    NodeOutput::out(vec![item])
}

#[cfg(test)]
mod tests {
    #[test]
    fn b07_prompt_snapshot() {
        let prompts = [
            ("write", include_str!("../../../../prompts/nodes/write.md")),
            ("summarise", include_str!("../../../../prompts/nodes/summarise.md")),
            ("classify", include_str!("../../../../prompts/nodes/classify.md")),
            ("extract", include_str!("../../../../prompts/nodes/extract.md")),
            ("translate", include_str!("../../../../prompts/nodes/translate.md")),
            ("decide", include_str!("../../../../prompts/nodes/decide.md")),
            ("agent", include_str!("../../../../prompts/nodes/agent.md")),
            ("agent_privacy", include_str!("../../../../prompts/nodes/agent_privacy.md")),
            ("describe_image", include_str!("../../../../prompts/nodes/describe_image.md")),
        ];
        let combined =
            prompts.into_iter().map(|(name, text)| format!("{name}: {}", text.trim())).collect::<Vec<_>>().join("\n");
        insta::assert_snapshot!(combined);
    }
}
