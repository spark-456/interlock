//! Runway text-to-video, text-to-speech and text-to-image task adapter.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use il_sdk::SafeHttp;
use il_sdk::registry::MediaGenFactory;
use il_sdk::services::{FileStore, MediaError, MediaGen, MediaKind, MediaRequest, MediaResult};
use serde_json::{Value, json};

struct Runway {
    http: SafeHttp,
    files: Arc<dyn FileStore>,
    base_url: String,
    key: String,
    video_model: String,
    voice_model: String,
    image_model: String,
}

impl Runway {
    fn body(&self, req: &MediaRequest) -> Result<(&'static str, Value), MediaError> {
        let p = &req.params;
        match req.kind {
            MediaKind::Video => {
                let duration = p.get("durationSec").and_then(Value::as_u64).unwrap_or(5);
                if !(2..=10).contains(&duration) {
                    return Err(MediaError::Other("video duration must be 2–10 seconds".into()));
                }
                let ratio = match p.get("aspect").and_then(Value::as_str).unwrap_or("16:9") {
                    "16:9" => "1280:720",
                    "9:16" => "720:1280",
                    _ => return Err(MediaError::Other("video aspect must be 16:9 or 9:16".into())),
                };
                Ok((
                    "text_to_video",
                    json!({"model": self.video_model, "promptText": req.prompt, "ratio": ratio, "duration": duration}),
                ))
            }
            MediaKind::Voice => {
                let mut body = json!({"model": self.voice_model, "promptText": req.prompt});
                if let Some(voice) = p.get("voice").and_then(Value::as_str) {
                    body["voice"] = json!({"type": "runway-preset", "presetId": voice});
                }
                Ok(("text_to_speech", body))
            }
            MediaKind::Image => {
                let ratio = match p.get("size").and_then(Value::as_str).unwrap_or("1024x1024") {
                    "1024x1024" => "1080:1080",
                    "1024x1792" => "1080:1920",
                    "1792x1024" => "1920:1080",
                    _ => return Err(MediaError::Other("unsupported image size".into())),
                };
                Ok(("text_to_image", json!({"model": self.image_model, "promptText": req.prompt, "ratio": ratio})))
            }
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/v1/{path}", self.base_url)
    }

    async fn json(&self, request: reqwest::RequestBuilder) -> Result<Value, MediaError> {
        let response = request
            .bearer_auth(&self.key)
            .header("X-Runway-Version", "2024-11-06")
            .send()
            .await
            .map_err(|e| MediaError::Provider(e.to_string()))?;
        if !response.status().is_success() {
            return Err(MediaError::Provider(format!("Runway HTTP {}", response.status())));
        }
        response.json().await.map_err(|e| MediaError::Provider(e.to_string()))
    }
}

#[async_trait]
impl MediaGen for Runway {
    async fn generate(&self, req: MediaRequest) -> Result<MediaResult, MediaError> {
        let (endpoint, body) = self.body(&req)?;
        let post = self.http.post(&self.url(endpoint)).map_err(|e| MediaError::Provider(e.to_string()))?;
        let created = self.json(post.json(&body)).await?;
        let id = created
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| MediaError::Provider("Runway task ID missing".into()))?;
        if !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            return Err(MediaError::Provider("invalid Runway task ID".into()));
        }
        let cost = credits(&created);
        for attempt in 0..60 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            let get =
                self.http.get(&self.url(&format!("tasks/{id}"))).map_err(|e| MediaError::Provider(e.to_string()))?;
            let task = self.json(get).await?;
            match task.get("status").and_then(Value::as_str) {
                Some("SUCCEEDED") => {
                    let url = task
                        .get("output")
                        .and_then(Value::as_array)
                        .and_then(|a| a.first())
                        .and_then(Value::as_str)
                        .ok_or_else(|| MediaError::Provider("Runway output URL missing".into()))?;
                    let download = self
                        .http
                        .get(url)
                        .map_err(|e| MediaError::Provider(e.to_string()))?
                        .send()
                        .await
                        .map_err(|e| MediaError::Provider(e.to_string()))?;
                    if !download.status().is_success() {
                        return Err(MediaError::Provider(format!("Runway download HTTP {}", download.status())));
                    }
                    let content_type =
                        download.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok());
                    let (name, mime) = match req.kind {
                        MediaKind::Video => ("generated.mp4", "video/mp4"),
                        MediaKind::Voice if content_type == Some("audio/wav") => ("generated.wav", "audio/wav"),
                        MediaKind::Voice => ("generated.mp3", "audio/mpeg"),
                        MediaKind::Image if content_type == Some("image/jpeg") => ("generated.jpg", "image/jpeg"),
                        MediaKind::Image if content_type == Some("image/webp") => ("generated.webp", "image/webp"),
                        MediaKind::Image => ("generated.png", "image/png"),
                    };
                    let bytes = download.bytes().await.map_err(|e| MediaError::Provider(e.to_string()))?;
                    let file = self
                        .files
                        .put_for_workspace(&req.workspace_id, name, mime, bytes)
                        .await
                        .map_err(|e| MediaError::Other(e.to_string()))?;
                    let credits =
                        credits(&task).or(cost).ok_or_else(|| MediaError::Provider("Runway cost missing".into()))?;
                    return Ok(MediaResult { file, cost_usd: credits * 0.01, provider: "runway".into() });
                }
                Some("FAILED" | "CANCELED") => return Err(MediaError::Provider("Runway generation failed".into())),
                Some("PENDING" | "RUNNING" | "THROTTLED") => {}
                _ => return Err(MediaError::Provider("unknown Runway task status".into())),
            }
        }
        Err(MediaError::Provider("Runway task timed out".into()))
    }
}

fn credits(task: &Value) -> Option<f64> {
    task.get("cost").or_else(|| task.get("estimatedCost"))?.get("credits")?.as_f64()
}

/// The configured model, or the default a user's own key runs with when the server has none.
fn model(section: &Value, field: &str, default: &str) -> String {
    section.get(field).and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or(default).into()
}

fn make(section: &Value, files: Arc<dyn FileStore>, key: Option<String>) -> Result<Arc<dyn MediaGen>, MediaError> {
    let key = key
        .or_else(|| std::env::var("RUNWAYML_API_SECRET").ok())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| MediaError::Other("set RUNWAYML_API_SECRET or connect a Runway key".into()))?;
    let http = SafeHttp::new().map_err(|e| MediaError::Other(e.to_string()))?;
    Ok(Arc::new(Runway {
        http,
        files,
        base_url: section
            .get("base_url")
            .and_then(Value::as_str)
            .unwrap_or("https://api.dev.runwayml.com")
            .trim_end_matches('/')
            .into(),
        key,
        video_model: model(section, "video_model", "gen4.5"),
        voice_model: model(section, "voice_model", "eleven_multilingual_v2"),
        image_model: model(section, "image_model", "gen4_image"),
    }))
}

inventory::submit! { MediaGenFactory { name: "runway", kinds: &[MediaKind::Video, MediaKind::Voice, MediaKind::Image], make } }

#[cfg(test)]
mod tests {
    use super::*;
    use il_sdk::testkit::FakeFiles;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn runway_adapters_submit_poll_download_and_report_cost() {
        let server = MockServer::start().await;
        let id = "17f20503-6c24-4c16-946b-35dbbce2af2f";
        let files = Arc::new(FakeFiles::default());
        let provider = Runway {
            http: SafeHttp::permissive_for_tests(),
            files: files.clone(),
            base_url: server.uri(),
            key: "test-key".into(),
            video_model: "video-test".into(),
            voice_model: "voice-test".into(),
            image_model: "image-test".into(),
        };
        Mock::given(method("GET"))
            .and(path(format!("/v1/tasks/{id}")))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "SUCCEEDED", "output": [format!("{}/file", server.uri())], "estimatedCost": {"credits": 25.0}
            })))
            .expect(3)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/file"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"fixture".to_vec()))
            .expect(3)
            .mount(&server)
            .await;
        for (kind, endpoint, body) in [
            (
                MediaKind::Video,
                "text_to_video",
                json!({"model":"video-test", "promptText":"sample", "ratio":"1280:720", "duration":5}),
            ),
            (
                MediaKind::Voice,
                "text_to_speech",
                json!({"model":"voice-test", "promptText":"sample", "voice":{"type":"runway-preset", "presetId":"Maya"}}),
            ),
            (
                MediaKind::Image,
                "text_to_image",
                json!({"model":"image-test", "promptText":"sample", "ratio":"1080:1080"}),
            ),
        ] {
            Mock::given(method("POST"))
                .and(path(format!("/v1/{endpoint}")))
                .and(header("authorization", "Bearer test-key"))
                .and(header("x-runway-version", "2024-11-06"))
                .and(body_partial_json(body))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": id})))
                .expect(1)
                .mount(&server)
                .await;
            let result = provider
                .generate(MediaRequest {
                    kind,
                    workspace_id: "ws_test".into(),
                    prompt: "sample".into(),
                    params: json!({"durationSec":5,"aspect":"16:9","voice":"Maya","size":"1024x1024"}),
                    recorded: false,
                    key: None,
                })
                .await
                .unwrap();
            assert_eq!(result.cost_usd, 0.25);
            assert_eq!(result.provider, "runway");
            assert_eq!(files.get(&result.file.id).await.unwrap().1.as_ref(), b"fixture");
        }
        server.verify().await;
    }
}
