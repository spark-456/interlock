//! Loads an API description through the SDK's file or SSRF-safe HTTP service.

use il_sdk::{SafeHttp, services::FileStore};

const MAX_SPEC_BYTES: usize = 2_000_000;

pub(super) async fn load(reference: &str, http: &SafeHttp, files: &dyn FileStore) -> Result<String, String> {
    let bytes = if reference.starts_with("http://") || reference.starts_with("https://") {
        let mut response = http
            .get(reference)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        if response.content_length().is_some_and(|size| size > MAX_SPEC_BYTES as u64) {
            return Err("spec exceeds 2 MB".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            if bytes.len() + chunk.len() > MAX_SPEC_BYTES {
                return Err("spec exceeds 2 MB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        bytes
    } else if reference.trim_start().starts_with("curl ") {
        reference.as_bytes().to_vec()
    } else {
        files.get(reference).await.map_err(|e| e.to_string())?.1.to_vec()
    };
    if bytes.len() > MAX_SPEC_BYTES {
        return Err("spec exceeds 2 MB".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
