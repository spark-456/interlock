//! Knowledge ingestion backed by SQLite, stored files and the model gateway.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use il_sdk::services::knowledge::{KnowledgeError, KnowledgeService, SearchHit, SourceInput};
use il_sdk::services::{FileStore, ModelGateway, StoreError};
use il_store::db::SqliteStore;
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};

use crate::chunk::{self, Chunk};
use crate::crawl::Tavily;
use crate::parse::{self, ParsedDocument, Section};
use crate::search;

/// Production knowledge service. All model and file operations use SDK traits.
#[derive(Clone)]
pub struct Knowledge {
    store: Arc<SqliteStore>,
    files: Arc<dyn FileStore>,
    models: Arc<dyn ModelGateway>,
    tavily: Option<Tavily>,
}

impl Knowledge {
    /// Builds the service; omit Tavily for offline or keyless workspaces.
    pub fn new(
        store: Arc<SqliteStore>,
        files: Arc<dyn FileStore>,
        models: Arc<dyn ModelGateway>,
        tavily: Option<Tavily>,
    ) -> Self {
        Self { store, files, models, tavily }
    }

    async fn prepare(
        &self,
        source: SourceInput,
    ) -> Result<Vec<(ParsedDocument, Option<String>, &'static str)>, KnowledgeError> {
        match source {
            SourceInput::Text { title, text } => Ok(vec![(plain(title, text), None, "knowledge")]),
            SourceInput::InstructionsText { title, text } => Ok(vec![(plain(title, text), None, "instructions")]),
            SourceInput::Url { url, crawl_limit } => {
                let tavily =
                    self.tavily.as_ref().ok_or_else(|| KnowledgeError::Other("Tavily is not configured".into()))?;
                let pages = tavily.crawl(&url, crawl_limit).await?;
                Ok(pages
                    .into_iter()
                    .map(|page| {
                        let doc = plain(page.url.clone(), page.text);
                        (doc, Some(page.url), "knowledge")
                    })
                    .collect())
            }
            SourceInput::File { file_id, name } => self.prepare_file(&file_id, &name, "knowledge").await,
            SourceInput::MarkedFile { file_id, name, role } => {
                let role = if role == "instructions" { "instructions" } else { "knowledge" };
                self.prepare_file(&file_id, &name, role).await
            }
        }
    }

    async fn prepare_file(
        &self,
        file_id: &str,
        name: &str,
        role: &'static str,
    ) -> Result<Vec<(ParsedDocument, Option<String>, &'static str)>, KnowledgeError> {
        let (file, bytes) = self.files.get(file_id).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        let document = if file.mime.starts_with("audio/") {
            let transcript =
                self.models.transcribe(&file, &bytes).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
            plain(name.into(), transcript.text)
        } else {
            let name = name.to_owned();
            tokio::task::spawn_blocking(move || parse::parse(&name, &file.mime, &bytes))
                .await
                .map_err(|e| KnowledgeError::Ingest(e.to_string()))??
        };
        Ok(vec![(document, None, role)])
    }

    async fn insert(
        &self,
        kb_id: &str,
        doc: ParsedDocument,
        url: Option<String>,
        role: &'static str,
    ) -> Result<String, KnowledgeError> {
        let source_id = il_sdk::id("src");
        let at = Utc::now();
        let chunks = if role == "knowledge" { chunk::split(&doc, at) } else { Vec::new() };
        let texts: Vec<String> = chunks.iter().map(|chunk| chunk.text.clone()).collect();
        let vectors = if texts.is_empty() {
            Vec::new()
        } else {
            self.models.embed(&texts).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?
        };
        if vectors.len() != chunks.len() {
            return Err(KnowledgeError::Ingest("embedding count mismatch".into()));
        }
        let dimension = vectors.first().map(Vec::len);
        if vectors.iter().any(|vector| vector.len() != dimension.unwrap_or(0)) || dimension == Some(0) {
            return Err(KnowledgeError::Ingest("embedding dimensions are inconsistent".into()));
        }
        let kb_id = kb_id.to_owned();
        let id = source_id.clone();
        let content = doc.sections.iter().map(|section| section.text.as_str()).collect::<Vec<_>>().join("\n");
        self.store
            .with_conn(move |conn| {
                let tx = conn.transaction().map_err(db)?;
                tx.execute(
                    "INSERT INTO kb_sources (id,kb_id,title,url,role,content,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![id, kb_id, doc.title, url, role, content, at.to_rfc3339()],
                )
                .map_err(db)?;
                ensure_vec_table(&tx, dimension)?;
                for (chunk, vector) in chunks.iter().zip(vectors.iter()) {
                    insert_chunk(&tx, &kb_id, &id, chunk, vector)?;
                }
                tx.commit().map_err(db)?;
                Ok(())
            })
            .await
            .map_err(knowledge)?;
        Ok(source_id)
    }
}

fn plain(title: String, text: String) -> ParsedDocument {
    ParsedDocument { title, sections: vec![Section { heading: None, text }] }
}

fn db(error: impl std::fmt::Display) -> StoreError {
    StoreError::Backend(error.to_string())
}
fn knowledge(error: StoreError) -> KnowledgeError {
    KnowledgeError::Other(error.to_string())
}

fn ensure_vec_table(tx: &rusqlite::Transaction<'_>, dimension: Option<usize>) -> Result<(), StoreError> {
    let Some(dimension) = dimension else {
        return Ok(());
    };
    if dimension > 8192 {
        return Err(StoreError::Backend("embedding exceeds 8192 dimensions".into()));
    }
    let current: Option<usize> =
        tx.query_row("SELECT dimensions FROM kb_vec_meta WHERE id=1", [], |row| row.get(0)).optional().map_err(db)?;
    match current {
        Some(previous) if previous == dimension => Ok(()),
        Some(_) => Err(StoreError::Backend("embedding dimension changed".into())),
        None => {
            tx.execute_batch(&format!("CREATE VIRTUAL TABLE kb_chunks_vec USING vec0(embedding float[{dimension}])"))
                .map_err(db)?;
            tx.execute("INSERT INTO kb_vec_meta (id,dimensions) VALUES (1,?1)", [dimension]).map_err(db)?;
            Ok(())
        }
    }
}

fn insert_chunk(
    tx: &rusqlite::Transaction<'_>,
    kb_id: &str,
    source_id: &str,
    chunk: &Chunk,
    vector: &[f32],
) -> Result<(), StoreError> {
    tx.execute(
        "INSERT INTO kb_chunks (kb_id,source_id,title,section,date,text) VALUES (?1,?2,?3,?4,?5,?6)",
        params![kb_id, source_id, chunk.title, chunk.section, chunk.date.to_rfc3339(), chunk.text],
    )
    .map_err(db)?;
    let row_id = tx.last_insert_rowid();
    let vector = serde_json::to_string(vector).map_err(db)?;
    tx.execute("INSERT INTO kb_chunks_vec (rowid,embedding) VALUES (?1,?2)", params![row_id, vector]).map_err(db)?;
    Ok(())
}

#[async_trait]
impl KnowledgeService for Knowledge {
    async fn owns_base(&self, workspace_id: &str, kb_id: &str) -> Result<bool, KnowledgeError> {
        let (workspace, id) = (workspace_id.to_owned(), kb_id.to_owned());
        self.store
            .with_conn(move |conn| {
                conn.query_row("SELECT 1 FROM kb_bases WHERE id=?1 AND workspace_id=?2", params![id, workspace], |_| {
                    Ok(true)
                })
                .optional()
                .map_err(db)
                .map(|found| found.unwrap_or(false))
            })
            .await
            .map_err(knowledge)
    }

    async fn create_base(&self, workspace_id: &str, name: &str) -> Result<String, KnowledgeError> {
        let id = il_sdk::id("kb");
        let (id_copy, workspace, name) = (id.clone(), workspace_id.to_owned(), name.to_owned());
        self.store
            .with_conn(move |conn| {
                conn.execute(
                    "INSERT INTO kb_bases (id,workspace_id,name,created_at) VALUES (?1,?2,?3,?4)",
                    params![id_copy, workspace, name, Utc::now().to_rfc3339()],
                )
                .map_err(db)?;
                Ok(())
            })
            .await
            .map_err(knowledge)?;
        Ok(id)
    }

    async fn add_sources(&self, kb_id: &str, sources: Vec<SourceInput>) -> Result<Vec<String>, KnowledgeError> {
        let id = kb_id.to_owned();
        let exists: bool = self
            .store
            .with_conn(move |conn| {
                conn.query_row("SELECT 1 FROM kb_bases WHERE id=?1", [&id], |_| Ok(true))
                    .optional()
                    .map_err(db)
                    .map(|found| found.unwrap_or(false))
            })
            .await
            .map_err(knowledge)?;
        if !exists {
            return Err(KnowledgeError::NotFound(kb_id.into()));
        }
        let mut ids = Vec::new();
        for source in sources {
            for (doc, url, role) in self.prepare(source).await? {
                ids.push(self.insert(kb_id, doc, url, role).await?);
            }
        }
        Ok(ids)
    }

    async fn search(&self, kb_id: &str, query: &str, top_k: usize) -> Result<Vec<SearchHit>, KnowledgeError> {
        let vector = self
            .models
            .embed(&[query.to_owned()])
            .await
            .map_err(|e| KnowledgeError::Other(e.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| KnowledgeError::Other("embedding model returned nothing".into()))?;
        search::hybrid(&self.store, kb_id, query, &vector, top_k).await
    }

    async fn parse(&self, file_id: &str, format: Option<&str>) -> Result<Value, KnowledgeError> {
        let (file, bytes) = self.files.get(file_id).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        let name = file.name.clone();
        let mime = format.unwrap_or(&file.mime).to_owned();
        let doc = tokio::task::spawn_blocking(move || parse::parse(&name, &mime, &bytes))
            .await
            .map_err(|e| KnowledgeError::Ingest(e.to_string()))??;
        Ok(
            json!({"title":doc.title,"sections":doc.sections.iter().map(|section| json!({"heading":section.heading,"text":section.text})).collect::<Vec<_>>() }),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use il_sdk::services::files::FileStore;
    use il_sdk::services::knowledge::{KnowledgeService, SourceInput};
    use il_sdk::testkit::{FakeFiles, FakeGateway};
    use il_store::db::SqliteStore;

    use super::Knowledge;

    #[tokio::test]
    async fn k01_ingest_and_hybrid_search_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("knowledge.sqlite");
        let files = Arc::new(FakeFiles::default());
        let models = Arc::new(FakeGateway::default());
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let service = Knowledge::new(store, files.clone(), models.clone(), None);
        let kb = service.create_base("ws_test", "Help").await.unwrap();
        assert!(service.owns_base("ws_test", &kb).await.unwrap());
        assert!(!service.owns_base("ws_other", &kb).await.unwrap());
        let ids = service
            .add_sources(
                &kb,
                vec![SourceInput::Text {
                    title: "Returns".into(),
                    text: "You may return an item within thirty days.".into(),
                }],
            )
            .await
            .unwrap();
        assert_eq!(ids.len(), 1);
        let hits = service.search(&kb, "return thirty days", 5).await.unwrap();
        assert_eq!(hits[0].source_id, ids[0]);
        assert_eq!(hits[0].title, "Returns");
        drop(service);
        let service = Knowledge::new(Arc::new(SqliteStore::open(path).unwrap()), files, models, None);
        assert_eq!(service.search(&kb, "return", 5).await.unwrap()[0].source_id, ids[0]);
    }

    #[tokio::test]
    async fn k01_instructions_are_stored_without_search_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(dir.path().join("knowledge.sqlite")).unwrap());
        let service =
            Knowledge::new(store.clone(), Arc::new(FakeFiles::default()), Arc::new(FakeGateway::default()), None);
        let kb = service.create_base("ws_test", "Voice memo").await.unwrap();
        service
            .add_sources(
                &kb,
                vec![SourceInput::InstructionsText { title: "Memo".into(), text: "Use a friendly tone".into() }],
            )
            .await
            .unwrap();
        assert!(service.search(&kb, "friendly", 5).await.unwrap().is_empty());
        let (role, content): (String, String) = store
            .with_conn(|conn| {
                conn.query_row("SELECT role,content FROM kb_sources LIMIT 1", [], |row| Ok((row.get(0)?, row.get(1)?)))
                    .map_err(|e| il_sdk::services::StoreError::Backend(e.to_string()))
            })
            .await
            .unwrap();
        assert_eq!(role, "instructions");
        assert_eq!(content, "Use a friendly tone");
    }

    #[tokio::test]
    async fn k01_audio_transcribes_to_instructions() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(dir.path().join("knowledge.sqlite")).unwrap());
        let files = Arc::new(FakeFiles::default());
        let models = Arc::new(FakeGateway::default());
        models.set_transcript("Use concise answers");
        let audio = files.put("memo.mp3", "audio/mpeg", vec![1, 2, 3].into()).await.unwrap();
        let service = Knowledge::new(store.clone(), files, models, None);
        let kb = service.create_base("ws_test", "Help").await.unwrap();
        service
            .add_sources(
                &kb,
                vec![SourceInput::MarkedFile {
                    file_id: audio.id,
                    name: "memo.mp3".into(),
                    role: "instructions".into(),
                }],
            )
            .await
            .unwrap();
        let (role, content): (String, String) = store
            .with_conn(|conn| {
                conn.query_row("SELECT role,content FROM kb_sources LIMIT 1", [], |row| Ok((row.get(0)?, row.get(1)?)))
                    .map_err(|e| il_sdk::services::StoreError::Backend(e.to_string()))
            })
            .await
            .unwrap();
        assert_eq!(role, "instructions");
        assert_eq!(content, "Use concise answers");
        assert!(service.search(&kb, "concise", 5).await.unwrap().is_empty());
    }
}
