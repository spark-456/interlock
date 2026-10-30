//! Media generation nodes and registered providers.

use std::sync::Arc;

use il_sdk::node::{NodeInput, NodeOutput};
use il_sdk::prelude::{CredentialKind, CredentialSpec, FileRef, NodeCtx, NodeError, cred};
use il_sdk::registry::media_gens;
use il_sdk::services::{FileStore, MediaError, MediaGen, MediaKey, MediaKind, MediaRequest, MediaResult};
use serde_json::Value;

pub mod image;
mod recorded;
mod runway;
pub mod video;
pub mod voice;

struct Selected {
    section: Value,
    files: Arc<dyn FileStore>,
    recorded: Arc<dyn MediaGen>,
    video: Arc<dyn MediaGen>,
    voice: Arc<dyn MediaGen>,
    image: Arc<dyn MediaGen>,
}

struct Unavailable(String);

#[async_trait::async_trait]
impl MediaGen for Unavailable {
    async fn generate(&self, _: MediaRequest) -> Result<MediaResult, MediaError> {
        Err(MediaError::Other(self.0.clone()))
    }
}

#[async_trait::async_trait]
impl MediaGen for Selected {
    async fn generate(&self, req: MediaRequest) -> Result<MediaResult, MediaError> {
        if !req.recorded
            && let Some(key) = req.key.clone()
        {
            let factory = media_gens()
                .find(|f| f.name == key.provider && f.kinds.contains(&req.kind))
                .ok_or(MediaError::NotConfigured(req.kind))?;
            return (factory.make)(&self.section, self.files.clone(), Some(key.secret))?.generate(req).await;
        }
        let provider = if req.recorded {
            &self.recorded
        } else {
            match req.kind {
                MediaKind::Video => &self.video,
                MediaKind::Voice => &self.voice,
                MediaKind::Image => &self.image,
            }
        };
        provider.generate(req).await
    }
}

/// Selects providers from `[providers.media]`; fake mode always uses fixtures.
pub fn from_config(config: &Value, files: Arc<dyn FileStore>, fake: bool) -> Result<Arc<dyn MediaGen>, MediaError> {
    let section = config.get("providers").and_then(|p| p.get("media")).unwrap_or(&Value::Null);
    let make = |name: &str, kind: MediaKind| -> Result<Arc<dyn MediaGen>, MediaError> {
        let factory =
            media_gens().find(|f| f.name == name && f.kinds.contains(&kind)).ok_or(MediaError::NotConfigured(kind))?;
        (factory.make)(section, files.clone(), None)
    };
    let recorded = make("recorded", MediaKind::Video)?;
    let select = |kind: MediaKind, field: &str| -> Arc<dyn MediaGen> {
        let name = if fake { "recorded" } else { section.get(field).and_then(Value::as_str).unwrap_or("recorded") };
        if name == "recorded" {
            recorded.clone()
        } else {
            make(name, kind).unwrap_or_else(|e| Arc::new(Unavailable(e.to_string())))
        }
    };
    Ok(Arc::new(Selected {
        video: select(MediaKind::Video, "video"),
        voice: select(MediaKind::Voice, "voice"),
        image: select(MediaKind::Image, "image"),
        recorded,
        section: section.clone(),
        files,
    }))
}

/// The user's own Runway key, saved on /connections; without it the step uses `[providers.media]`.
pub(super) fn own_key() -> CredentialSpec {
    CredentialSpec { optional: true, ..cred("runway", "runway", CredentialKind::ApiKey, "Runway API key", &[]) }
}

/// The key bound on the step, if any. Read only through `ctx.credential()`, never from item data.
async fn bound_key(ctx: &NodeCtx) -> Result<Option<MediaKey>, NodeError> {
    let spec = own_key();
    if !ctx.credential_ids.contains_key(&spec.key) {
        return Ok(None);
    }
    let credential = ctx.credential(&spec.key).await?;
    let secret = credential.secret["api_key"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| NodeError::Failed(format!("the {} is empty", spec.label)))?;
    Ok(Some(MediaKey { provider: spec.app, secret: secret.into() }))
}

pub(super) fn prompt(settings: &Value, key: &str) -> Result<String, NodeError> {
    let value = settings.get(key).and_then(Value::as_str).unwrap_or_default().trim();
    if value.is_empty() {
        return Err(NodeError::Failed(format!("{key} must not be empty")));
    }
    Ok(value.to_owned())
}

/// Asks the provider for one piece of media, with the step's own key when one is bound.
pub(super) async fn generate(
    ctx: &NodeCtx,
    kind: MediaKind,
    prompt: String,
    params: Value,
) -> Result<MediaResult, NodeError> {
    let request = MediaRequest {
        kind,
        workspace_id: ctx.run.workspace_id.clone(),
        prompt,
        params,
        recorded: ctx.run.test || ctx.run.workspace_id.starts_with("ws_demo"),
        key: bound_key(ctx).await?,
    };
    ctx.media().generate(request).await.map_err(|e| NodeError::Failed(e.to_string()))
}

/// The input item with the generated file, its provider and its cost.
pub(super) fn output(input: NodeInput, kind: MediaKind, file: FileRef, provider: &str, cost_usd: f64) -> NodeOutput {
    let mut item = input.main().first().cloned().unwrap_or_default();
    let label = match kind {
        MediaKind::Video => "video",
        MediaKind::Voice => "voice",
        MediaKind::Image => "image",
    };
    // Voice is saved as `audio`, the file the merge and transcribe steps read.
    let file_key = if kind == MediaKind::Voice { "audio" } else { label };
    item.files.insert(file_key.into(), file);
    item.json[label] = serde_json::json!({"provider": provider, "costUsd": cost_usd});
    let mut output = NodeOutput::out(vec![item]);
    output.cost_usd = cost_usd;
    output
}

pub(super) async fn execute(
    ctx: &mut NodeCtx,
    input: NodeInput,
    kind: MediaKind,
    key: &str,
) -> Result<NodeOutput, NodeError> {
    let prompt = prompt(&ctx.settings, key)?;
    ctx.emit_progress(0.0, "Generation requested");
    let result = generate(ctx, kind, prompt, ctx.settings.clone()).await?;
    ctx.emit_progress(100.0, "Generated media saved");
    Ok(output(input, kind, result.file, &result.provider, result.cost_usd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use il_sdk::services::{FileStore, MediaKind, Store};
    use il_sdk::testkit::{FakeFiles, NodeHarness};
    use serde_json::json;

    #[tokio::test]
    async fn recorded_provider_saves_playable_fixture_bytes() {
        let files = Arc::new(FakeFiles::default());
        let media = from_config(
            &json!({"providers":{"media":{"video":"runway","voice":"runway","image":"runway"}}}),
            files.clone(),
            true,
        )
        .unwrap();
        for kind in [MediaKind::Video, MediaKind::Voice, MediaKind::Image] {
            let result = media
                .generate(MediaRequest {
                    kind,
                    workspace_id: "ws_demo_1".into(),
                    prompt: "sample".into(),
                    params: json!({}),
                    recorded: true,
                    key: Some(MediaKey { provider: "runway".into(), secret: "user-key".into() }),
                })
                .await
                .unwrap();
            let (_, bytes) = files.get(&result.file.id).await.unwrap();
            assert!(!bytes.is_empty());
            assert_eq!(result.provider, "recorded");
            assert_eq!(result.cost_usd, 0.0);
        }
    }

    #[tokio::test]
    async fn generation_nodes_return_files_progress_and_cost() {
        let video =
            NodeHarness::new(video::AiGenerateVideo).settings(json!({"prompt": "A blue sky", "durationSec": 5}));
        let result = video.run().await.unwrap();
        assert_eq!(result.ports["out"][0].files["video"].mime, "video/mp4");
        assert_eq!(result.cost_usd, 0.0);
        assert_eq!(video.progress.events.lock().unwrap().len(), 2);

        let voice = NodeHarness::new(voice::AiGenerateVoice).settings(json!({"text": "Hello"}));
        let result = voice.run().await.unwrap();
        assert_eq!(result.ports["out"][0].files["audio"].mime, "audio/mpeg");

        let image = NodeHarness::new(image::AiGenerateImage).settings(json!({"prompt": "A blue sky"}));
        let result = image.run().await.unwrap();
        assert_eq!(result.ports["out"][0].files["image"].mime, "image/png");
    }

    #[tokio::test]
    async fn a_bound_key_is_passed_on_and_an_empty_one_fails() {
        let video = NodeHarness::new(video::AiGenerateVideo)
            .settings(json!({"prompt": "A blue sky"}))
            .credential("runway", "cred_runway");
        let saved = |key: &str| il_sdk::services::Credential {
            id: "cred_runway".into(),
            app: "runway".into(),
            secret: json!({"api_key": key}),
            meta: json!({}),
        };
        video.fakes.store.put_credential("ws_test", &saved("key_123")).await.unwrap();
        let result = video.run().await.unwrap();
        assert_eq!(result.ports["out"][0].json["video"]["provider"], "runway");

        video.fakes.store.put_credential("ws_test", &saved(" ")).await.unwrap();
        assert!(video.run().await.unwrap_err().to_string().contains("Runway API key is empty"));
    }

    #[tokio::test]
    async fn own_keys_override_the_server_provider_except_in_recorded_runs() {
        let files = Arc::new(FakeFiles::default());
        // The server only has fixtures; a loopback base URL makes the Runway call fail before any network.
        let media = from_config(
            &json!({"providers":{"media":{"video":"recorded","base_url":"http://127.0.0.1:9"}}}),
            files,
            false,
        )
        .unwrap();
        let request = |provider: &str, recorded: bool| MediaRequest {
            kind: MediaKind::Video,
            workspace_id: "ws_1".into(),
            prompt: "sample".into(),
            params: json!({}),
            recorded,
            key: Some(MediaKey { provider: provider.into(), secret: "user-key".into() }),
        };
        let err = media.generate(request("runway", false)).await.unwrap_err().to_string();
        assert!(err.contains("blocked"), "{err}");
        assert!(matches!(media.generate(request("nope", false)).await, Err(MediaError::NotConfigured(_))));
        assert_eq!(media.generate(request("runway", true)).await.unwrap().provider, "recorded");
    }
}
