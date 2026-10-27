//! API keys: hashed at rest, revealed once, revoked by id.

mod routes;

use chrono::{DateTime, Utc};
use il_core::dto::ApiKeySummary;
use il_sdk::services::{Store, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// API key persistence errors.
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("storage: {0}")]
    Store(#[from] StoreError),
    #[error("key not found")]
    NotFound,
}

#[derive(Serialize, Deserialize)]
struct KeyRecord {
    workspace_id: String,
    name: String,
    hash: String,
    revoked: bool,
    #[serde(default)]
    last_used_at: Option<DateTime<Utc>>,
}

/// Manages published API keys in the shared store.
pub struct KeyManager {
    store: Arc<dyn Store>,
}

impl KeyManager {
    /// Uses the shared store without persisting raw keys.
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    /// Creates a key and returns the raw value exactly once.
    pub async fn create(&self, workspace_id: &str, name: &str) -> Result<(String, String), KeyError> {
        let id = ulid::Ulid::new().to_string();
        let raw = format!("ilk_{id}_{}{}", ulid::Ulid::new(), ulid::Ulid::new());
        let record = KeyRecord {
            workspace_id: workspace_id.into(),
            name: name.into(),
            hash: hash(&raw),
            revoked: false,
            last_used_at: None,
        };
        self.store
            .kv_put(
                &format!("publish_api_key:{id}"),
                &serde_json::to_value(record).map_err(|e| StoreError::Backend(e.to_string()))?,
            )
            .await?;
        let index_key = format!("publish_api_keys:{workspace_id}");
        let mut ids: Vec<String> = self
            .store
            .kv_get(&index_key)
            .await?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| StoreError::Backend(e.to_string()))?
            .unwrap_or_default();
        ids.push(id.clone());
        self.store
            .kv_put(&index_key, &serde_json::to_value(ids).map_err(|e| StoreError::Backend(e.to_string()))?)
            .await?;
        Ok((id, raw))
    }

    /// Lists keys for one workspace without revealing secrets.
    pub async fn list(&self, workspace_id: &str) -> Result<Vec<ApiKeySummary>, KeyError> {
        let index_key = format!("publish_api_keys:{workspace_id}");
        let ids: Vec<String> = self
            .store
            .kv_get(&index_key)
            .await?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| StoreError::Backend(e.to_string()))?
            .unwrap_or_default();
        let mut keys = Vec::new();
        for id in ids {
            let Some(value) = self.store.kv_get(&format!("publish_api_key:{id}")).await? else { continue };
            let record: KeyRecord = serde_json::from_value(value).map_err(|e| StoreError::Backend(e.to_string()))?;
            if record.workspace_id == workspace_id {
                keys.push(ApiKeySummary {
                    id,
                    name: record.name,
                    revoked: record.revoked,
                    last_used_at: record.last_used_at,
                });
            }
        }
        Ok(keys)
    }

    /// Returns the workspace only for a valid, active key.
    pub async fn authenticate(&self, raw: &str) -> Result<Option<String>, KeyError> {
        let Some(id) = raw.strip_prefix("ilk_").and_then(|rest| rest.split_once('_')).map(|(id, _)| id) else {
            return Ok(None);
        };
        if id.len() != 26 {
            return Ok(None);
        }
        let Some(value) = self.store.kv_get(&format!("publish_api_key:{id}")).await? else { return Ok(None) };
        let mut record: KeyRecord = serde_json::from_value(value).map_err(|e| StoreError::Backend(e.to_string()))?;
        let candidate = hash(raw);
        let same = candidate.len() == record.hash.len()
            && candidate.as_bytes().iter().zip(record.hash.as_bytes()).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0;
        if record.revoked || !same {
            return Ok(None);
        }
        record.last_used_at = Some(Utc::now());
        self.store
            .kv_put(
                &format!("publish_api_key:{id}"),
                &serde_json::to_value(&record).map_err(|e| StoreError::Backend(e.to_string()))?,
            )
            .await?;
        Ok(Some(record.workspace_id))
    }

    /// Revokes a key belonging to `workspace_id`.
    pub async fn revoke(&self, workspace_id: &str, id: &str) -> Result<(), KeyError> {
        let key = format!("publish_api_key:{id}");
        let value = self.store.kv_get(&key).await?.ok_or(KeyError::NotFound)?;
        let mut record: KeyRecord = serde_json::from_value(value).map_err(|e| StoreError::Backend(e.to_string()))?;
        if record.workspace_id != workspace_id {
            return Err(KeyError::NotFound);
        }
        record.revoked = true;
        self.store.kv_put(&key, &serde_json::to_value(record).map_err(|e| StoreError::Backend(e.to_string()))?).await?;
        Ok(())
    }
}

fn hash(raw: &str) -> String {
    format!("{:x}", Sha256::digest(raw.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::KeyManager;
    use il_sdk::testkit::MemoryStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn p01_api_key_is_revealed_once_and_revocable() {
        let manager = KeyManager::new(Arc::new(MemoryStore::default()));
        let (id, raw) = manager.create("ws_a", "Demo").await.unwrap();
        assert_eq!(manager.authenticate(&raw).await.unwrap().as_deref(), Some("ws_a"));
        assert!(manager.authenticate(&format!("{raw}x")).await.unwrap().is_none());
        manager.revoke("ws_a", &id).await.unwrap();
        assert!(manager.authenticate(&raw).await.unwrap().is_none());
    }
}
