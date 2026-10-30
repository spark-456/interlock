//! In-memory `FileStore`.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;
use il_core::FileRef;

use crate::services::files::{FileError, FileStore};

/// Files kept in memory; `local_path` writes a temp copy.
#[derive(Default)]
pub struct FakeFiles {
    files: Mutex<HashMap<String, (FileRef, Bytes)>>,
    owners: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl FileStore for FakeFiles {
    async fn put(&self, name: &str, mime: &str, bytes: Bytes) -> Result<FileRef, FileError> {
        let id = crate::id("file");
        let r = FileRef {
            id: id.clone(),
            mime: mime.into(),
            bytes: bytes.len() as u64,
            name: name.into(),
            preview_url: Some(format!("/api/files/{id}")),
        };
        self.files.lock().map_err(|_| FileError::Backend("poisoned".into()))?.insert(id, (r.clone(), bytes));
        Ok(r)
    }

    async fn put_for_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        mime: &str,
        bytes: Bytes,
    ) -> Result<FileRef, FileError> {
        let file = self.put(name, mime, bytes).await?;
        self.owners
            .lock()
            .map_err(|_| FileError::Backend("poisoned".into()))?
            .insert(file.id.clone(), workspace_id.into());
        Ok(file)
    }

    async fn delete_workspace_files(&self, workspace_id: &str) -> Result<(), FileError> {
        let mut owners = self.owners.lock().map_err(|_| FileError::Backend("poisoned".into()))?;
        let mut files = self.files.lock().map_err(|_| FileError::Backend("poisoned".into()))?;
        owners.retain(|id, owner| {
            if owner == workspace_id {
                files.remove(id);
                false
            } else {
                true
            }
        });
        Ok(())
    }

    async fn owner(&self, id: &str) -> Result<Option<String>, FileError> {
        Ok(self.owners.lock().map_err(|_| FileError::Backend("poisoned".into()))?.get(id).cloned())
    }

    async fn get(&self, id: &str) -> Result<(FileRef, Bytes), FileError> {
        self.files
            .lock()
            .map_err(|_| FileError::Backend("poisoned".into()))?
            .get(id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.into()))
    }

    async fn local_path(&self, id: &str) -> Result<std::path::PathBuf, FileError> {
        let (r, bytes) = self.get(id).await?;
        let path = std::env::temp_dir().join(format!("il-fake-{}-{}", r.id, r.name));
        tokio::fs::write(&path, &bytes).await.map_err(|e| FileError::Backend(e.to_string()))?;
        Ok(path)
    }
}
