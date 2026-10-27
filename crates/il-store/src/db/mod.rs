//! SQLite connection, extension registration, and ordered migrations.

use std::{
    path::Path,
    sync::{Arc, Mutex, Once},
};

use il_sdk::services::store::{StoreError, StoreResult};
use include_dir::{Dir, include_dir};
use rusqlite::{Connection, OptionalExtension, params};

static MIGRATIONS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../migrations");
static VEC_INIT: Once = Once::new();
type ExtensionEntry =
    unsafe extern "C" fn(*mut rusqlite::ffi::sqlite3, *mut *mut i8, *const rusqlite::ffi::sqlite3_api_routines) -> i32;

/// A serialized SQLite connection. Every operation runs on Tokio's blocking pool.
#[derive(Clone)]
pub struct SqliteStore {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) key: Option<[u8; 32]>,
}

impl SqliteStore {
    /// Opens a database, enables WAL and foreign keys, and applies embedded migrations.
    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        VEC_INIT.call_once(|| unsafe {
            // sqlite-vec is statically linked; register its entrypoint before opening connections.
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<*const (), ExtensionEntry>(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        });
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(backend)?;
        }
        let mut conn = Connection::open(path).map_err(backend)?;
        conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(backend)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").map_err(backend)?;
        migrate(&mut conn)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)), key: crate::secrets::key_from_env()? })
    }

    /// Sets a master key directly, useful for isolated tests and explicit server wiring.
    pub fn with_master_key(mut self, key: [u8; 32]) -> Self {
        self.key = Some(key);
        self
    }

    /// Runs one SQLite operation without blocking an async executor thread.
    pub(crate) async fn with_conn<T, F>(&self, f: F) -> StoreResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> StoreResult<T> + Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = conn.lock().map_err(|_| StoreError::Backend("database lock poisoned".into()))?;
            f(&mut guard)
        })
        .await
        .map_err(backend)?
    }
}

fn migrate(conn: &mut Connection) -> StoreResult<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS il_migrations (name TEXT PRIMARY KEY)").map_err(backend)?;
    let mut files: Vec<_> =
        MIGRATIONS.files().filter(|f| f.path().extension().is_some_and(|ext| ext == "sql")).collect();
    files.sort_by_key(|f| f.path());
    for file in files {
        let name = file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| StoreError::Backend("invalid migration name".into()))?;
        let applied: Option<String> = conn
            .query_row("SELECT name FROM il_migrations WHERE name=?1", [name], |row| row.get(0))
            .optional()
            .map_err(backend)?;
        if applied.is_some() {
            continue;
        }
        let sql = file.contents_utf8().ok_or_else(|| StoreError::Backend(format!("migration {name} is not UTF-8")))?;
        let tx = conn.transaction().map_err(backend)?;
        tx.execute_batch(sql).map_err(backend)?;
        tx.execute("INSERT INTO il_migrations (name) VALUES (?1)", params![name]).map_err(backend)?;
        tx.commit().map_err(backend)?;
    }
    Ok(())
}

pub(crate) fn backend(error: impl std::fmt::Display) -> StoreError {
    StoreError::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b01_migrations_wal_vec_and_fts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.sqlite");
        let store = SqliteStore::open(&path).unwrap();
        let conn = store.conn.lock().unwrap();
        let wal: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
        assert_eq!(wal, "wal");
        let version: String = conn.query_row("SELECT vec_version()", [], |r| r.get(0)).unwrap();
        assert!(!version.is_empty());
        conn.execute_batch("CREATE VIRTUAL TABLE b01_fts USING fts5(content); DROP TABLE b01_fts;").unwrap();
        drop(conn);
        SqliteStore::open(path).unwrap();
    }
}
