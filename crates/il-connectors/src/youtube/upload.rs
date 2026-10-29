//! `app.youtube.upload` — Upload video (YouTube).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// Upload video (YouTube) node.
#[derive(Debug, Default)]
pub struct AppYoutubeUpload;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.youtube.upload",
        Category::App,
        "Upload video (YouTube)",
        "Uploads a video to YouTube (private by default).",
    )
    .icon("video")
    .app("youtube")
    .external()
    .status(NodeStatus::Ready)
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/youtube.upload"],
    ))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "video",
            "title"
        ],
        "properties": {
            "video": {
                "type": "string",
                "title": "Video",
                "default": "{{ item.files.video }}",
                "x-widget": "file"
            },
            "title": {
                "type": "string",
                "title": "Title",
                "x-widget": "expression"
            },
            "description": {
                "type": "string",
                "title": "Description",
                "x-widget": "longtext"
            },
            "privacy": {
                "type": "string",
                "title": "Privacy",
                "enum": [
                    "private",
                    "unlisted"
                ],
                "default": "private",
                "x-widget": "select"
            },
            "tags": {
                "type": "array",
                "title": "Tags",
                "items": {
                    "type": "string"
                },
                "default": []
            }
        }
    }))
    .example(
        "Private upload",
        json!({
            "video": "{{ item.files.clip_16x9 }}",
            "title": "{{ item.title }}",
            "privacy": "private"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppYoutubeUpload {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let file_id =
            ctx.settings["video"].as_str().ok_or_else(|| NodeError::Failed("missing video file id".into()))?;
        let title = ctx.settings["title"].as_str().ok_or_else(|| NodeError::Failed("missing title".into()))?;
        let privacy = ctx.settings["privacy"].as_str().unwrap_or("private");
        if !matches!(privacy, "private" | "unlisted") {
            return Err(NodeError::Failed("upload privacy must be private or unlisted".into()));
        }
        let key = format!("youtube:upload:{}", ctx.dedupe_key());
        if let Some(saved) = ctx.services.store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))? {
            let mut item = input.main().first().cloned().unwrap_or_default();
            item.json["youtube"] = saved;
            return Ok(NodeOutput::out(vec![item]));
        }
        let (file, bytes) = ctx.files().get(file_id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        if !file.mime.starts_with("video/") {
            return Err(NodeError::Failed("file is not a video".into()));
        }
        let token = crate::oauth::google_token(&ctx.services, &ctx.run.workspace_id, &ctx.credential_ids).await?;
        let token = token.as_str();
        #[cfg(test)]
        let base = ctx.settings["_base"].as_str().unwrap_or("https://www.googleapis.com");
        #[cfg(not(test))]
        let base = "https://www.googleapis.com";
        let url = format!("{base}/upload/youtube/v3/videos?uploadType=resumable&part=snippet,status");
        let init = ctx.http().post(&url).map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token).header("X-Upload-Content-Type",&file.mime)
            .header("X-Upload-Content-Length",bytes.len().to_string())
            .json(&json!({"snippet":{"title":title,"description":ctx.settings["description"].as_str().unwrap_or(""),
                "tags":ctx.settings["tags"].as_array().cloned().unwrap_or_default()},"status":{"privacyStatus":privacy}}))
            .send().await.map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status().map_err(|e| NodeError::Failed(e.to_string()))?;
        let location = init
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| NodeError::Failed("YouTube did not return an upload location".into()))?
            .to_owned();
        let result: Value = ctx
            .http()
            .request(reqwest::Method::PUT, &location)
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .header("Content-Type", &file.mime)
            .body(bytes)
            .send()
            .await
            .map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .json()
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        ctx.services.store.kv_put(&key, &result).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["youtube"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppYoutubeUpload)) }

#[cfg(test)]
mod tests {
    use il_sdk::{
        prelude::*,
        services::{Credential, FileStore},
        testkit::NodeHarness,
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppYoutubeUpload);
    }

    #[tokio::test]
    async fn c01_youtube_upload_starts_private_resumable_session() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/upload/youtube/v3/videos"))
            .respond_with(ResponseTemplate::new(200).insert_header("location", format!("{}/resume", server.uri())))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/resume"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"video1"})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppYoutubeUpload).credential("google", "cred_1");
        let file = harness.fakes.files.put("clip.mp4", "video/mp4", vec![1, 2, 3].into()).await.unwrap();
        let harness = harness.settings(json!({"video":file.id,"title":"Demo","_base":server.uri()}));
        harness.fakes.store.add_credential(Credential {
            id: "cred_1".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["youtube"]["id"], "video1");
        let requests = server.received_requests().await.unwrap();
        let init: Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(init["status"]["privacyStatus"], "private");
        assert_eq!(requests[1].body, vec![1, 2, 3]);
    }
}
