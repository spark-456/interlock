//! `ai.guardrails` — route an answer through configured safety rules.

use il_sdk::prelude::*;
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelRole};

/// Safety rules node.
#[derive(Debug, Default)]
pub struct AiGuardrails;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.guardrails", Category::Ai, "Safety rules",
        "Checks an answer against configured rules and routes it to pass or blocked, with an optional human hand-off.",
    )
    .icon("shield")
    .status(NodeStatus::Ready)
    .outputs(&["pass", "blocked", "handoff"])
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type":"object",
        "required":["rules"],
        "properties":{
            "input":{"type":"string","title":"Answer to check","default":"{{ item.answer }}","x-widget":"expression"},
            "rules":{"type":"array","title":"Rules","minItems":1,"items":{
                "type":"object","required":["name","description","action"],
                "properties":{
                    "name":{"type":"string","title":"Name"},
                    "description":{"type":"string","title":"Rule","x-widget":"longtext"},
                    "action":{"type":"string","title":"If matched","enum":["block","handoff"],"x-widget":"select"}
                }
            }},
            "model":{"type":"string","title":"Model","enum":["architect","agent","fast"],"default":"fast","x-widget":"model-role"}
        }
    }))
    .example("Refund policy", json!({
        "input":"{{ item.answer }}",
        "rules":[{"name":"large_refund","description":"Refunds over $100 require a person to review the answer.","action":"handoff"}]
    }))
});

#[async_trait]
impl NodeExecutor for AiGuardrails {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let rules = ctx.settings["rules"].as_array().ok_or_else(|| NodeError::Failed("rules must be a list".into()))?;
        let mut names = vec!["none"];
        for rule in rules {
            let name = rule["name"]
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| NodeError::Failed("rule name is required".into()))?;
            if names.contains(&name) {
                return Err(NodeError::Failed("rule names must be unique".into()));
            }
            names.push(name);
        }
        let role = match ctx.settings["model"].as_str().unwrap_or("fast") {
            "fast" => ModelRole::Fast,
            "agent" => ModelRole::Agent,
            "architect" => ModelRole::Architect,
            _ => return Err(NodeError::Failed("unknown model role".into())),
        };
        let answer = ctx
            .settings
            .get("input")
            .cloned()
            .or_else(|| input.main().first().and_then(|item| item.json.get("answer")).cloned())
            .unwrap_or(Value::Null);
        let answer = answer.as_str().map(str::to_owned).unwrap_or_else(|| answer.to_string());
        // The person's own message tells whose details an answer may repeat (their own email is fine).
        let asked = input.main().first().and_then(|item| item.json["message"].as_str()).unwrap_or_default();
        let request = ChatRequest {
            messages: vec![
                ChatMessage::system(format!(
                    "You check an assistant's answer against rules. A rule matches only when the answer clearly \
                     breaks it; if you are unsure, or the rule needs facts you can't see, it does not match. Details \
                     the person gave about themselves are theirs, not another customer's. Return JSON with the name \
                     of the first matching rule, or none. Never choose an action. Rules: {}",
                    serde_json::to_string(rules).map_err(|e| NodeError::Failed(e.to_string()))?
                )),
                ChatMessage::user(if asked.is_empty() {
                    answer
                } else {
                    format!("The person wrote:\n{asked}\n\nThe answer to check:\n{answer}")
                }),
            ],
            json_schema: Some(
                json!({"type":"object","required":["rule"],"properties":{"rule":{"type":"string","enum":names}},"additionalProperties":false}),
            ),
            // The same answer must get the same verdict; sampling made small models flag good answers.
            temperature: Some(0.0),
            ..Default::default()
        };
        let response = ctx.models().chat(role, request).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let parsed = response.json().map_err(|e| NodeError::Failed(e.to_string()))?;
        let matched = parsed["rule"].as_str().ok_or_else(|| NodeError::Failed("model returned no rule".into()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        if matched == "none" {
            item.json["guardrails"] = json!({"decision":"pass"});
            return Ok(NodeOutput::port("pass", vec![item]));
        }
        let rule = rules
            .iter()
            .find(|rule| rule["name"] == matched)
            .ok_or_else(|| NodeError::Failed("model returned an unknown rule".into()))?;
        let action = rule["action"].as_str().ok_or_else(|| NodeError::Failed("rule action is required".into()))?;
        item.json["guardrails"] =
            json!({"decision":"blocked","rule":matched,"action":action,"handoff":action == "handoff"});
        Ok(NodeOutput::port(if action == "handoff" { "handoff" } else { "blocked" }, vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiGuardrails)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiGuardrails);
    }

    #[tokio::test]
    async fn n01_passes_when_no_rule_matches() {
        let harness = NodeHarness::new(super::AiGuardrails)
            .settings(json!({"rules":[{"name":"unsafe","description":"No unsafe claims","action":"block"}]}))
            .input_json(vec![json!({"answer":"Hello"})]);
        harness.fakes.models.push_json(json!({"rule":"none"}));
        assert_eq!(harness.run().await.unwrap().ports["pass"][0].json["guardrails"]["decision"], "pass");
    }

    #[tokio::test]
    async fn n01_hands_off_matched_rule() {
        let harness = NodeHarness::new(super::AiGuardrails)
            .settings(json!({"rules":[{"name":"refund","description":"Refund over $100","action":"handoff"}]}))
            .input_json(vec![json!({"answer":"Refund $150"})]);
        harness.fakes.models.push_json(json!({"rule":"refund"}));
        let output = harness.run().await.unwrap();
        assert_eq!(output.ports["handoff"][0].json["guardrails"]["handoff"], true);
        assert!(harness.fakes.models.calls()[0].1.json_schema.is_some());
    }

    #[tokio::test]
    async fn n01_rejects_unknown_rule() {
        let harness = NodeHarness::new(super::AiGuardrails)
            .settings(json!({"rules":[{"name":"refund","description":"Refund over $100","action":"block"}]}));
        harness.fakes.models.push_json(json!({"rule":"unknown"}));
        assert!(harness.run().await.unwrap_err().to_string().contains("unknown rule"));
    }
}
