//! `app.drive.upload` — upload an item file into a configured folder.

use il_sdk::prelude::*;

/// Uploads one item file to Google Drive.
#[derive(Debug, Default)]
pub struct AppDriveUpload;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.drive.upload",
        Category::App,
        "Upload file (Google Drive)",
        "Uploads an item file into a selected Drive folder.",
    )
    .icon("upload")
    .app("drive")
    .external()
    .status(NodeStatus::Ready)
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/drive"],
    ))
    .settings_schema(json!({"type":"object","required":["folderId"],"properties":{
        "folderId":{"type":"string","title":"Folder ID"},
        "file":{"type":"string","title":"Item file key","default":"attachment"}
    }}))
    .example("Save invoice", json!({"folderId":"drive-folder-id","file":"attachment"}))
});

#[async_trait]
impl NodeExecutor for AppDriveUpload {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }
    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let folder = ctx.settings["folderId"]
            .as_str()
            .filter(|id| !id.is_empty() && id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')))
            .ok_or_else(|| NodeError::Failed("invalid folderId".into()))?;
        let key = ctx.settings["file"].as_str().unwrap_or("attachment");
        let mut item = input.main().first().cloned().ok_or_else(|| NodeError::Failed("missing item".into()))?;
        let reference = item.files.get(key).ok_or_else(|| NodeError::Failed(format!("missing {key} file")))?;
        let (file, bytes) = ctx.files().get(&reference.id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        reqwest::header::HeaderValue::from_str(&file.mime)
            .map_err(|_| NodeError::Failed("invalid file MIME type".into()))?;
        let dedupe = format!("drive:upload:{}:{}", ctx.dedupe_key(), file.id);
        if let Some(saved) = ctx.services.store.kv_get(&dedupe).await.map_err(|e| NodeError::Failed(e.to_string()))? {
            item.json["drive"] = saved;
            return Ok(NodeOutput::out(vec![item]));
        }
        let credential = ctx.credential("google").await?;
        let token = credential.secret["access_token"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("missing Google access token".into()))?;
        #[cfg(test)]
        let base = ctx.settings["_base"].as_str().unwrap_or("https://www.googleapis.com");
        #[cfg(not(test))]
        let base = "https://www.googleapis.com";
        let metadata = json!({"name":file.name,"parents":[folder]});
        let root = format!("{base}/upload/drive/v3/files");
        let response = if bytes.len() <= 5 * 1024 * 1024 {
            let boundary = il_sdk::id("il");
            let mut body = format!(
                "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n\
                --{boundary}\r\nContent-Type: {}\r\n\r\n",
                file.mime
            )
            .into_bytes();
            body.extend_from_slice(&bytes);
            body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
            ctx.http()
                .post(&format!("{root}?uploadType=multipart&fields=id,name,webViewLink"))
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .bearer_auth(token)
                .header(reqwest::header::CONTENT_TYPE, format!("multipart/related; boundary={boundary}"))
                .body(body)
                .send()
                .await
                .map_err(|e| NodeError::Retryable(e.to_string()))?
        } else {
            let started = ctx
                .http()
                .post(&format!("{root}?uploadType=resumable&fields=id,name,webViewLink"))
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .bearer_auth(token)
                .header(reqwest::header::CONTENT_TYPE, "application/json; charset=UTF-8")
                .header("X-Upload-Content-Type", &file.mime)
                .header("X-Upload-Content-Length", bytes.len().to_string())
                .json(&metadata)
                .send()
                .await
                .map_err(|e| NodeError::Retryable(e.to_string()))?
                .error_for_status()
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            let location = started
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| NodeError::Failed("Drive upload session has no location".into()))?;
            ctx.http()
                .request(reqwest::Method::PUT, location)
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .bearer_auth(token)
                .header(reqwest::header::CONTENT_TYPE, &file.mime)
                .body(bytes)
                .send()
                .await
                .map_err(|e| NodeError::Retryable(e.to_string()))?
        };
        let result: Value = response
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .json()
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        if result["id"].as_str().is_none() {
            return Err(NodeError::Failed("Drive upload response has no file id".into()));
        }
        ctx.services.store.kv_put(&dedupe, &result).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        item.json["drive"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppDriveUpload)) }

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

    #[tokio::test]
    async fn c06_drive_upload_sends_file_and_configured_folder() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/upload/drive/v3/files"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"drive_1","name":"invoice.txt"})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppDriveUpload)
            .settings(json!({"folderId":"folder_1","file":"attachment","_base":server.uri()}))
            .credential("google", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        let file = harness.fakes.files.put("invoice.txt", "text/plain", "hello".into()).await.unwrap();
        let mut item = Item::json(json!({"folderId":"wrong"}));
        item.files.insert("attachment".into(), file);
        let output = harness.input(vec![item]).run().await.unwrap();
        assert_eq!(output.ports["out"][0].json["drive"]["id"], "drive_1");
        let requests = server.received_requests().await.unwrap();
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(body.contains("\"parents\":[\"folder_1\"]"));
        assert!(body.contains("hello"));
        assert!(!body.contains("wrong"));
    }

    #[tokio::test]
    async fn c06_drive_upload_uses_resumable_session_above_five_megabytes() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/upload/drive/v3/files"))
            .respond_with(ResponseTemplate::new(200).insert_header("Location", format!("{}/session", server.uri())))
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/session"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"large_1"})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppDriveUpload)
            .settings(json!({"folderId":"folder_1","_base":server.uri()}))
            .credential("google", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        let bytes = vec![7; 5 * 1024 * 1024 + 1];
        let file = harness.fakes.files.put("large.bin", "application/octet-stream", bytes.into()).await.unwrap();
        let mut item = Item::json(json!({}));
        item.files.insert("attachment".into(), file);
        let output = harness.input(vec![item]).run().await.unwrap();
        assert_eq!(output.ports["out"][0].json["drive"]["id"], "large_1");
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[1].body.len(), 5 * 1024 * 1024 + 1);
    }
}
