//! `trigger.drive.new_file` — poll the Drive change feed.

use std::{collections::HashSet, sync::Arc, time::Duration};

use chrono::DateTime;
use il_sdk::prelude::*;
use il_sdk::{
    TriggerCtx, TriggerHandle,
    poll::{Poller, spawn_polling},
};

/// Starts a run when a new Drive file appears.
#[derive(Debug, Default)]
pub struct TriggerDriveNewFile;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.drive.new_file",
        Category::Trigger,
        "New file (Google Drive)",
        "Starts a flow for each new file in Drive or a selected folder.",
    )
    .icon("folder")
    .app("drive")
    .trigger()
    .status(NodeStatus::Ready)
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/drive"],
    ))
    .settings_schema(json!({"type":"object","properties":{
        "folderId":{"type":"string","title":"Folder ID","description":"Leave empty to watch all Drive files."},
        "pollMinutes":{"type":"integer","title":"Check every (min)","default":1,"minimum":1,"maximum":60}
    }}))
    .example("Watch invoices", json!({"folderId":"invoices-folder-id","pollMinutes":1}))
});

#[async_trait]
impl NodeExecutor for TriggerDriveNewFile {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }
    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        Ok(NodeOutput::out(input.main().to_vec()))
    }
    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for TriggerDriveNewFile {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError> {
        let minutes = ctx.settings["pollMinutes"].as_u64().unwrap_or(1).clamp(1, 60);
        Ok(spawn_polling(ctx, "trigger.drive.new_file", Duration::from_secs(minutes * 60), Arc::new(DrivePoller)))
    }
}

struct DrivePoller;

#[async_trait]
impl Poller for DrivePoller {
    async fn poll(&self, ctx: &TriggerCtx, cursor: Option<Value>) -> Result<(Vec<Item>, Option<Value>), NodeError> {
        let token = crate::oauth::google_token(&ctx.services, &ctx.workspace_id, &ctx.credential_ids).await?;
        let token = token.as_str();
        #[cfg(test)]
        let base = ctx.settings["_base"].as_str().unwrap_or("https://www.googleapis.com");
        #[cfg(not(test))]
        let base = "https://www.googleapis.com";
        let folder = ctx.settings["folderId"].as_str().filter(|s| !s.is_empty());
        if folder.is_some_and(|id| !id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))) {
            return Err(NodeError::Failed("invalid folderId".into()));
        }
        let Some(cursor) = cursor else {
            let response: Value = ctx
                .services
                .http
                .get(&format!("{base}/drive/v3/changes/startPageToken"))
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| NodeError::Retryable(e.to_string()))?
                .error_for_status()
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .json()
                .await
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            let page_token = response["startPageToken"]
                .as_str()
                .ok_or_else(|| NodeError::Failed("Drive start token missing".into()))?;
            return Ok((Vec::new(), Some(json!({"token":page_token,"since":ctx.services.clock.now().to_rfc3339()}))));
        };
        let mut page =
            cursor["token"].as_str().ok_or_else(|| NodeError::Failed("invalid Drive cursor".into()))?.to_owned();
        let since = cursor["since"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .ok_or_else(|| NodeError::Failed("invalid Drive cursor time".into()))?;
        let mut items = Vec::new();
        let mut seen = HashSet::new();
        for _ in 0..100 {
            let mut url =
                url::Url::parse(&format!("{base}/drive/v3/changes")).map_err(|e| NodeError::Failed(e.to_string()))?;
            url.query_pairs_mut().append_pair("pageToken", &page).append_pair("pageSize", "1000")
                .append_pair("fields", "nextPageToken,newStartPageToken,changes(fileId,removed,file(id,name,mimeType,createdTime,parents,webViewLink,trashed))");
            let response: Value = ctx
                .services
                .http
                .get(url.as_str())
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| NodeError::Retryable(e.to_string()))?
                .error_for_status()
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .json()
                .await
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            for change in response["changes"].as_array().into_iter().flatten() {
                if change["removed"] == true {
                    continue;
                }
                let file = &change["file"];
                if file["trashed"] == true || file["mimeType"] == "application/vnd.google-apps.folder" {
                    continue;
                }
                if folder.is_some_and(|id| {
                    !file["parents"].as_array().is_some_and(|parents| parents.iter().any(|p| p == id))
                }) {
                    continue;
                }
                let Some(created) = file["createdTime"].as_str().and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                else {
                    continue;
                };
                if created <= since {
                    continue;
                }
                let Some(id) = file["id"].as_str().or_else(|| change["fileId"].as_str()) else { continue };
                if !seen.insert(id.to_owned()) {
                    continue;
                }
                let mut item = file.as_object().cloned().unwrap_or_default();
                item.insert("fileId".into(), json!(id));
                items.push(Item::json(Value::Object(item)));
            }
            if let Some(next) = response["nextPageToken"].as_str() {
                page = next.to_owned();
                continue;
            }
            let next = response["newStartPageToken"]
                .as_str()
                .ok_or_else(|| NodeError::Failed("Drive next token missing".into()))?;
            return Ok((items, Some(json!({"token":next,"since":ctx.services.clock.now().to_rfc3339()}))));
        }
        Err(NodeError::Retryable("Drive change feed has too many pages".into()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerDriveNewFile)) }

#[cfg(test)]
mod tests {
    use il_sdk::{
        TriggerCtx,
        poll::Poller,
        services::Credential,
        testkit::{FakeRunner, FakeServices},
    };
    use serde_json::json;
    use std::{collections::BTreeMap, sync::Arc, time::Duration};
    use tokio_util::sync::CancellationToken;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn c06_drive_new_file_uses_cursor_and_ignores_edits() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/drive/v3/changes/startPageToken"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"startPageToken":"t1"})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/drive/v3/changes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "changes":[
                    {"fileId":"new","file":{"id":"new","name":"invoice.pdf","mimeType":"application/pdf",
                        "createdTime":"2026-10-26T09:30:00Z","parents":["folder_1"]}},
                    {"fileId":"edited","file":{"id":"edited","name":"old.pdf","mimeType":"application/pdf",
                        "createdTime":"2026-10-26T08:00:00Z","parents":["folder_1"]}}
                ],"newStartPageToken":"t2"
            })))
            .mount(&server)
            .await;
        let fakes = FakeServices::default();
        fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        let ctx = TriggerCtx {
            flow_id: "flow".into(),
            workspace_id: "ws".into(),
            node_id: "n".into(),
            settings: json!({"folderId":"folder_1","_base":server.uri()}),
            credential_ids: BTreeMap::from([("google".into(), "cred".into())]),
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            cancel: CancellationToken::new(),
        };
        let (initial, cursor) = super::DrivePoller.poll(&ctx, None).await.unwrap();
        assert!(initial.is_empty());
        assert_eq!(cursor.as_ref().unwrap()["token"], "t1");
        fakes.clock.advance(Duration::from_secs(3600));
        let (items, cursor) = super::DrivePoller.poll(&ctx, cursor).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].json["fileId"], "new");
        assert_eq!(cursor.unwrap()["token"], "t2");
    }
}
