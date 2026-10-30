//! Checked-in media returned in test runs.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use il_sdk::registry::MediaGenFactory;
use il_sdk::services::{FileStore, MediaError, MediaGen, MediaKind, MediaRequest, MediaResult};
use serde_json::Value;

struct Recorded {
    files: Arc<dyn FileStore>,
}

#[async_trait]
impl MediaGen for Recorded {
    async fn generate(&self, req: MediaRequest) -> Result<MediaResult, MediaError> {
        let (name, mime, bytes): (&str, &str, &[u8]) = match req.kind {
            MediaKind::Video => ("recorded.mp4", "video/mp4", include_bytes!("fixtures/clip.mp4")),
            MediaKind::Voice => ("recorded.mp3", "audio/mpeg", include_bytes!("fixtures/voice.mp3")),
            MediaKind::Image => ("recorded.png", "image/png", include_bytes!("fixtures/image.png")),
        };
        let file = self
            .files
            .put(name, mime, Bytes::copy_from_slice(bytes))
            .await
            .map_err(|e| MediaError::Other(e.to_string()))?;
        Ok(MediaResult { file, cost_usd: 0.0, provider: "recorded".into() })
    }
}

fn make(_: &Value, files: Arc<dyn FileStore>, _: Option<String>) -> Result<Arc<dyn MediaGen>, MediaError> {
    Ok(Arc::new(Recorded { files }))
}

inventory::submit! { MediaGenFactory { name: "recorded", kinds: &[MediaKind::Video, MediaKind::Voice, MediaKind::Image], make } }
