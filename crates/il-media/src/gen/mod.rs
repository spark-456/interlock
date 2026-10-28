//! Media generation nodes and registered providers.

use std::sync::Arc;

use il_sdk::registry::media_gens;
use il_sdk::services::{FileStore, MediaError, MediaGen, MediaKind, MediaRequest};
use serde_json::Value;

pub mod image;
mod recorded;
mod runway;
pub mod video;
pub mod voice;

struct Selected {
    recorded: Arc<dyn MediaGen>,
    video: Arc<dyn MediaGen>,
    voice: Arc<dyn MediaGen>,
    image: Arc<dyn MediaGen>,
}

struct Unavailable(String);

#[async_trait::async_trait]
impl MediaGen for Unavailable {
    async fn generate(&self, _: MediaRequest) -> Result<il_sdk::services::MediaResult, MediaError> {
        Err(MediaError::Other(self.0.clone()))
    }
}

#[async_trait::async_trait]
impl MediaGen for Selected {
    async fn generate(&self, req: MediaRequest) -> Result<il_sdk::services::MediaResult, MediaError> {
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
        (factory.make)(section, files.clone())
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
    }))
}

pub(super) fn prompt(settings: &Value, key: &str) -> Result<String, il_sdk::node::NodeError> {
    let value = settings.get(key).and_then(Value::as_str).unwrap_or_default().trim();
    if value.is_empty() {
        return Err(il_sdk::node::NodeError::Failed(format!("{key} must not be empty")));
    }
    Ok(value.to_owned())
}

pub(super) async fn execute(
    ctx: &mut il_sdk::ctx::NodeCtx,
    input: il_sdk::node::NodeInput,
    kind: MediaKind,
    key: &str,
) -> Result<il_sdk::node::NodeOutput, il_sdk::node::NodeError> {
    let request = MediaRequest {
        kind,
        workspace_id: ctx.run.workspace_id.clone(),
        prompt: prompt(&ctx.settings, key)?,
        params: ctx.settings.clone(),
        recorded: ctx.run.test || ctx.run.workspace_id.starts_with("ws_demo"),
    };
    ctx.emit_progress(0.0, "Generation requested");
    let result = ctx.media().generate(request).await.map_err(|e| il_sdk::node::NodeError::Failed(e.to_string()))?;
    ctx.emit_progress(100.0, "Generated media saved");
    let mut item = input.main().first().cloned().unwrap_or_default();
    let label = match kind {
        MediaKind::Video => "video",
        MediaKind::Voice => "voice",
        MediaKind::Image => "image",
    };
    // Voice is saved as `audio`, the file the merge and transcribe steps read.
    let file_key = if kind == MediaKind::Voice { "audio" } else { label };
    item.files.insert(file_key.into(), result.file);
    item.json[label] = serde_json::json!({"provider": result.provider, "costUsd": result.cost_usd});
    let mut output = il_sdk::node::NodeOutput::out(vec![item]);
    output.cost_usd = result.cost_usd;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use il_sdk::services::{FileStore, MediaKind};
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
}
