//! `output.respond` — Respond.
//!
//! Produces the caller response from a rendered answer and optional sources.

use il_sdk::prelude::*;

/// Respond node.
#[derive(Debug, Default)]
pub struct OutputRespond;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "output.respond",
        Category::Output,
        "Respond",
        "Sends the answer back to the chat, API or MCP caller (streaming when possible).",
    )
    .icon("reply")
    .status(NodeStatus::Ready)
    .outputs(&[])
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "body": {
                "type": "string",
                "title": "Answer",
                "default": "{{ item.answer }}",
                "x-widget": "expression"
            },
            "format": {
                "type": "string",
                "title": "Format",
                "enum": [
                    "text",
                    "json"
                ],
                "default": "text",
                "x-widget": "select"
            },
            "includeSources": {
                "type": "boolean",
                "title": "Include sources",
                "default": true
            }
        }
    }))
    .example(
        "Answer with sources",
        json!({
            "body": "{{ item.answer }}",
            "includeSources": true
        }),
    )
});

#[async_trait]
impl NodeExecutor for OutputRespond {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let mut item = input.main().first().cloned().unwrap_or_default();
        let body =
            ctx.settings.get("body").cloned().or_else(|| item.json.get("answer").cloned()).unwrap_or(Value::Null);
        let answer = if ctx.settings["format"].as_str().unwrap_or("text") == "json" {
            body
        } else {
            Value::String(body.as_str().map(str::to_owned).unwrap_or_else(|| body.to_string()))
        };
        let sources = if ctx.settings["includeSources"].as_bool().unwrap_or(true) {
            item.json.get("sources").cloned().unwrap_or_else(|| json!([]))
        } else {
            json!([])
        };
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["answer"] = answer;
        item.json["sources"] = sources;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(OutputRespond)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::OutputRespond);
    }

    #[tokio::test]
    async fn b08_respond_keeps_answer_and_sources() {
        let harness = NodeHarness::new(super::OutputRespond)
            .settings(json!({"body":"{{ item.draft }}","includeSources":true}))
            .input_json(vec![json!({"draft":"Hello","sources":[{"title":"Guide"}]})]);
        let item = &harness.run().await.unwrap().ports["out"][0];
        assert_eq!(item.json["answer"], "Hello");
        assert_eq!(item.json["sources"][0]["title"], "Guide");
    }

    #[tokio::test]
    async fn b08_respond_preserves_json_body() {
        let harness = NodeHarness::new(super::OutputRespond)
            .settings(json!({"body":"{{ item.result }}","format":"json","includeSources":false}))
            .input_json(vec![json!({"result":{"ok":true},"sources":[{"title":"Hidden"}]})]);
        let item = &harness.run().await.unwrap().ports["out"][0];
        assert_eq!(item.json["answer"]["ok"], true);
        assert_eq!(item.json["sources"], json!([]));
    }
}
