//! Binary files referenced by items.

use async_trait::async_trait;
use bytes::Bytes;
use il_core::FileRef;

/// File errors.
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("file storage: {0}")]
    Backend(String),
}

/// Stores files and hands out `FileRef`s with preview URLs.
#[async_trait]
pub trait FileStore: Send + Sync {
    /// Saves bytes and returns a reference.
    async fn put(&self, name: &str, mime: &str, bytes: Bytes) -> Result<FileRef, FileError>;
    /// Saves a file owned by a workspace for scoped cleanup.
    async fn put_for_workspace(
        &self,
        _workspace_id: &str,
        name: &str,
        mime: &str,
        bytes: Bytes,
    ) -> Result<FileRef, FileError> {
        self.put(name, mime, bytes).await
    }
    /// Deletes files owned by a workspace.
    async fn delete_workspace_files(&self, _workspace_id: &str) -> Result<(), FileError> {
        Ok(())
    }
    /// Reads a file.
    async fn get(&self, id: &str) -> Result<(FileRef, Bytes), FileError>;
    /// A local path for child processes such as ffmpeg (copying to a temp file if needed).
    async fn local_path(&self, id: &str) -> Result<std::path::PathBuf, FileError>;
}
