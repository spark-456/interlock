//! `FileStore` on disk under `data/files/` with preview URLs.

use std::path::PathBuf;

use async_trait::async_trait;
use bytes::Bytes;
use il_core::FileRef;
use il_sdk::services::files::{FileError, FileStore};
use rusqlite::{OptionalExtension, params};

use crate::db::{SqliteStore, backend};

/// Disk-backed files with metadata in the same SQLite database.
pub struct DiskFileStore {
    store: SqliteStore,
    root: PathBuf,
}

impl DiskFileStore {
    /// Creates the file directory and binds it to a SQLite store.
    pub async fn new(store: SqliteStore, root: impl Into<PathBuf>) -> Result<Self, FileError> {
        let root = root.into();
        tokio::fs::create_dir_all(&root).await.map_err(file_backend)?;
        Ok(Self { store, root })
    }

    async fn metadata(&self, id: &str) -> Result<Option<(FileRef, PathBuf)>, FileError> {
        let id = id.to_owned();
        let root = self.root.clone();
        self.store
            .with_conn(move |c| {
                let row: Option<(String, String, String, u64, String)> = c
                    .query_row("SELECT id,name,mime,bytes,path FROM fs_files WHERE id=?1", [&id], |r| {
                        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                    })
                    .optional()
                    .map_err(backend)?;
                Ok(row.map(|(id, name, mime, bytes, path)| {
                    let preview_url = Some(format!("/api/files/{id}"));
                    (FileRef { id, name, mime, bytes, preview_url }, root.join(path))
                }))
            })
            .await
            .map_err(file_backend)
    }
}

#[async_trait]
impl FileStore for DiskFileStore {
    async fn put(&self, name: &str, mime: &str, bytes: Bytes) -> Result<FileRef, FileError> {
        let id = format!("file_{}", ulid::Ulid::new());
        let path_name = format!("{id}.blob");
        let path = self.root.join(&path_name);
        tokio::fs::write(&path, &bytes).await.map_err(file_backend)?;
        let result = FileRef {
            id: id.clone(),
            name: name.to_owned(),
            mime: mime.to_owned(),
            bytes: bytes.len() as u64,
            preview_url: Some(format!("/api/files/{id}")),
        };
        let record = result.clone();
        let save = self
            .store
            .with_conn(move |c| {
                c.execute(
                    "INSERT INTO fs_files (id,name,mime,bytes,path,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                    params![
                        record.id,
                        record.name,
                        record.mime,
                        record.bytes,
                        path_name,
                        chrono::Utc::now().to_rfc3339()
                    ],
                )
                .map_err(backend)?;
                Ok(())
            })
            .await;
        if let Err(error) = save {
            let _ = tokio::fs::remove_file(path).await;
            return Err(file_backend(error));
        }
        Ok(result)
    }

    async fn get(&self, id: &str) -> Result<(FileRef, Bytes), FileError> {
        let (reference, path) = self.metadata(id).await?.ok_or_else(|| FileError::NotFound(id.to_owned()))?;
        let bytes = tokio::fs::read(path).await.map_err(file_backend)?;
        Ok((reference, Bytes::from(bytes)))
    }

    async fn local_path(&self, id: &str) -> Result<PathBuf, FileError> {
        let (_, path) = self.metadata(id).await?.ok_or_else(|| FileError::NotFound(id.to_owned()))?;
        tokio::fs::metadata(&path).await.map_err(file_backend)?;
        Ok(path)
    }
}

fn file_backend(error: impl std::fmt::Display) -> FileError {
    FileError::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn b01_files_round_trip_with_preview_and_local_path() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(dir.path().join("store.sqlite")).unwrap();
        let files = DiskFileStore::new(store, dir.path().join("files")).await.unwrap();
        let reference = files.put("hello.txt", "text/plain", Bytes::from_static(b"hello")).await.unwrap();
        assert_eq!(reference.preview_url.as_deref(), Some(format!("/api/files/{}", reference.id).as_str()));
        assert_eq!(files.get(&reference.id).await.unwrap().1, Bytes::from_static(b"hello"));
        assert!(files.local_path(&reference.id).await.unwrap().exists());
        assert!(matches!(files.get("missing").await, Err(FileError::NotFound(_))));
    }
}
