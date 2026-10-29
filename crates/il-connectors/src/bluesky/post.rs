//! `app.bluesky.post` — Post (Bluesky).
//!
//! Posts text and an optional image or video blob through the account's PDS.

use il_sdk::prelude::*;

/// Post (Bluesky) node.
#[derive(Debug, Default)]
pub struct AppBlueskyPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.bluesky.post",
        Category::App,
        "Post (Bluesky)",
        "Posts text with an optional image or video to Bluesky.",
    )
    .icon("send")
    .status(NodeStatus::Ready)
    .app("bluesky")
    .external()
    .credential(cred("bluesky", "bluesky", CredentialKind::AppPassword, "Bluesky app password", &[]))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "text"
        ],
        "properties": {
            "text": {
                "type": "string",
                "title": "Post",
                "x-widget": "longtext"
            },
            "media": {
                "type": "string",
                "title": "Media",
                "x-widget": "expression"
            },
            "altText": {
                "type": "string",
                "title": "Alt text"
            }
        }
    }))
    .example(
        "Promo post",
        json!({
            "text": "{{ item.post }}",
            "media": "{{ item.files.clip_1x1 }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppBlueskyPost {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let text = ctx.settings["text"].as_str().ok_or_else(|| NodeError::Failed("text is required".into()))?;
        let credential = ctx.credential("bluesky").await?;
        let identifier = credential.secret["identifier"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("Bluesky identifier missing".into()))?;
        let password = credential.secret["app_password"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("Bluesky app password missing".into()))?;
        let pds = credential.meta["pds_url"].as_str().unwrap_or("https://bsky.social");
        let media = &ctx.settings["media"];
        let alt = ctx.settings["altText"].as_str().unwrap_or("");
        // Once per run and item, so a retry or a resumed run never posts twice.
        let post = super::post(ctx, pds, identifier, password, text, media, alt);
        crate::rest::once(ctx, "bluesky:post", async { post.await.map(Value::from) }).await?;
        Ok(NodeOutput::out(input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppBlueskyPost)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::{Credential, FileStore};
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppBlueskyPost);
    }

    #[tokio::test]
    async fn c02_bluesky_posts_video_blob() {
        let server = MockServer::start().await;
        for (path_name, response) in [
            ("/xrpc/com.atproto.server.createSession", json!({"accessJwt":"token","did":"did:plc:test"})),
            (
                "/xrpc/com.atproto.repo.uploadBlob",
                json!({"blob":{"$type":"blob","ref":{"$link":"abc"},"mimeType":"video/mp4","size":5}}),
            ),
            ("/xrpc/com.atproto.repo.createRecord", json!({"uri":"at://did:plc:test/app.bsky.feed.post/1"})),
        ] {
            Mock::given(method("POST"))
                .and(path(path_name))
                .respond_with(ResponseTemplate::new(200).set_body_json(response))
                .expect(1)
                .mount(&server)
                .await;
        }
        let harness = NodeHarness::new(super::AppBlueskyPost).credential("bluesky", "cred");
        let file = harness.fakes.files.put("clip.mp4", "video/mp4", "video".into()).await.unwrap();
        let harness = harness.settings(json!({"text":"hello","media":file.id,"altText":"clip"}));
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "bluesky".into(),
            secret: json!({"identifier":"alice.test","app_password":"pw"}),
            meta: json!({"pds_url":server.uri()}),
        });
        assert!(harness.run().await.is_ok());
        let requests = server.received_requests().await.unwrap();
        let create = requests.iter().find(|request| request.url.path().ends_with("createRecord")).unwrap();
        let body: serde_json::Value = serde_json::from_slice(&create.body).unwrap();
        assert_eq!(body["record"]["embed"]["$type"], "app.bsky.embed.video");
    }
}
