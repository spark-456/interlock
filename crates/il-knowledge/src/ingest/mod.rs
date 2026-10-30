//! Knowledge ingestion backed by SQLite, stored files and the model gateway.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use il_sdk::services::knowledge::{BaseCopy, KnowledgeError, KnowledgeService, SearchHit, SourceInput};
use il_sdk::services::{FileStore, ModelGateway, StoreError};
use il_store::db::SqliteStore;
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};

use crate::chunk::{self, Chunk};
use crate::crawl::Tavily;
use crate::parse::{self, ParsedDocument, Section};
use crate::search;

mod copy;

/// Documents ready to chunk: each with its source URL and role (`knowledge`, `instructions`).
type Prepared = Vec<(ParsedDocument, Option<String>, &'static str)>;

/// Texts per embedding call, and calls in flight. A crawled site has hundreds of chunks and one call
/// can take many seconds, so a page-by-page loop outlasted the step timeout.
const EMBED_BATCH: usize = 128;
const EMBED_PARALLEL: usize = 4;
/// When a query's embedding hasn't come back by then, a second request is sent and the first answer wins:
/// the embedding model sometimes takes many seconds, and a query costs almost nothing.
const EMBED_HEDGE: Duration = Duration::from_millis(1500);
/// After this, a search goes on by keywords alone rather than keep the person waiting.
const EMBED_WAIT: Duration = Duration::from_secs(4);

/// Production knowledge service. All model and file operations use SDK traits.
#[derive(Clone)]
pub struct Knowledge {
    store: Arc<SqliteStore>,
    files: Arc<dyn FileStore>,
    models: Arc<dyn ModelGateway>,
    tavily: Option<Tavily>,
    /// Pages for `.example` sites, which can never exist online: the demo templates' help centers.
    recorded: Option<PathBuf>,
}

impl Knowledge {
    /// Builds the service; omit Tavily for offline or keyless workspaces.
    pub fn new(
        store: Arc<SqliteStore>,
        files: Arc<dyn FileStore>,
        models: Arc<dyn ModelGateway>,
        tavily: Option<Tavily>,
    ) -> Self {
        Self { store, files, models, tavily, recorded: None }
    }

    /// Serves `.example` sites from the Markdown pages in `dir` instead of crawling them.
    pub fn with_recorded_sites(mut self, dir: PathBuf) -> Self {
        self.recorded = Some(dir);
        self
    }

    /// Embeds `texts` in batches, a few calls at a time, keeping their order.
    async fn embed_all(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, KnowledgeError> {
        let batches: Vec<&[String]> = texts.chunks(EMBED_BATCH).collect();
        let mut vectors = Vec::with_capacity(texts.len());
        for group in batches.chunks(EMBED_PARALLEL) {
            let calls = group.iter().map(|batch| self.models.embed(batch));
            for batch in
                futures::future::try_join_all(calls).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?
            {
                vectors.extend(batch);
            }
        }
        Ok(vectors)
    }

    async fn prepare(&self, source: SourceInput) -> Result<Prepared, KnowledgeError> {
        match source {
            SourceInput::Text { title, text } => Ok(vec![(plain(title, text), None, "knowledge")]),
            SourceInput::InstructionsText { title, text } => Ok(vec![(plain(title, text), None, "instructions")]),
            SourceInput::Url { url, crawl_limit } if self.recorded.is_some() && is_example(&url) => {
                let dir = self.recorded.clone().unwrap_or_default();
                tokio::task::spawn_blocking(move || recorded_pages(&dir, &url, crawl_limit))
                    .await
                    .map_err(|e| KnowledgeError::Ingest(e.to_string()))?
            }
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

    async fn prepare_file(&self, file_id: &str, name: &str, role: &'static str) -> Result<Prepared, KnowledgeError> {
        let (file, bytes) = self.files.get(file_id).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        // Flow steps name a file by its id; the stored name is what people recognise in citations.
        let name = if name == file_id { file.name.clone() } else { name.to_owned() };
        let name = name.as_str();
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
        (doc, url, role): (ParsedDocument, Option<String>, &'static str),
        chunks: Vec<Chunk>,
        vectors: Vec<Vec<f32>>,
        at: chrono::DateTime<Utc>,
    ) -> Result<String, KnowledgeError> {
        let source_id = il_sdk::id("src");
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

fn is_example(url: &str) -> bool {
    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h == "example" || h.ends_with(".example")))
        == Some(true)
}

/// The recorded Markdown pages as if crawled from `url`, one page per file.
fn recorded_pages(dir: &std::path::Path, url: &str, limit: u32) -> Result<Prepared, KnowledgeError> {
    let read = |e: std::io::Error| KnowledgeError::Ingest(format!("recorded site: {e}"));
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(read)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .take(limit as usize)
        .map(|path| {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let doc = parse::parse(&name, "text/markdown", &std::fs::read(&path).map_err(read)?)?;
            let stem = name.trim_end_matches(".md");
            Ok((doc, Some(format!("{}/{stem}", url.trim_end_matches('/'))), "knowledge"))
        })
        .collect()
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
            tx.execute_batch(&format!(
                "CREATE VIRTUAL TABLE kb_chunks_vec USING vec0(embedding float[{dimension}] distance_metric=cosine)"
            ))
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
    async fn export_base(&self, workspace_id: &str, kb_id: &str) -> Result<Option<BaseCopy>, KnowledgeError> {
        copy::export_base(&self.store, workspace_id, kb_id).await.map_err(knowledge)
    }

    async fn list_bases(&self, workspace_id: &str) -> Result<Vec<Value>, KnowledgeError> {
        let workspace = workspace_id.to_owned();
        self.store
            .with_conn(move |conn| {
                let sql = "SELECT b.id, b.name, b.created_at, COUNT(s.id) FROM kb_bases b                            LEFT JOIN kb_sources s ON s.kb_id = b.id WHERE b.workspace_id = ?1                            GROUP BY b.id ORDER BY b.created_at DESC";
                let mut stmt = conn.prepare(sql).map_err(|e| StoreError::Backend(e.to_string()))?;
                let rows = stmt
                    .query_map(params![workspace], |r| {
                        Ok(json!({
                            "id": r.get::<_, String>(0)?,
                            "name": r.get::<_, String>(1)?,
                            "createdAt": r.get::<_, String>(2)?,
                            "sources": r.get::<_, i64>(3)?,
                        }))
                    })
                    .map_err(|e| StoreError::Backend(e.to_string()))?;
                rows.collect::<Result<Vec<_>, _>>().map_err(|e| StoreError::Backend(e.to_string()))
            })
            .await
            .map_err(|e| KnowledgeError::Other(e.to_string()))
    }

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
        let mut docs = Vec::new();
        for source in sources {
            docs.extend(self.prepare(source).await?);
        }
        let at = Utc::now();
        let chunks: Vec<Vec<Chunk>> = docs
            .iter()
            .map(|(doc, _, role)| if *role == "knowledge" { chunk::split(doc, at) } else { Vec::new() })
            .collect();
        let texts: Vec<String> = chunks.iter().flatten().map(|chunk| chunk.text.clone()).collect();
        let mut vectors = self.embed_all(&texts).await?.into_iter();
        let mut ids = Vec::new();
        for (doc, chunks) in docs.into_iter().zip(chunks) {
            let own: Vec<Vec<f32>> = vectors.by_ref().take(chunks.len()).collect();
            ids.push(self.insert(kb_id, doc, chunks, own, at).await?);
        }
        Ok(ids)
    }

    async fn search(&self, kb_id: &str, query: &str, top_k: usize) -> Result<Vec<SearchHit>, KnowledgeError> {
        let texts = [query.to_owned()];
        let second = async {
            tokio::time::sleep(EMBED_HEDGE).await;
            self.models.embed(&texts).await
        };
        let embedded = futures::future::select_ok([self.models.embed(&texts), Box::pin(second)]);
        let vector = match tokio::time::timeout(EMBED_WAIT, embedded).await {
            Ok(found) => {
                let (vectors, _) = found.map_err(|e| KnowledgeError::Other(e.to_string()))?;
                Some(
                    vectors
                        .into_iter()
                        .next()
                        .ok_or_else(|| KnowledgeError::Other("embedding model returned nothing".into()))?,
                )
            }
            Err(_) => None,
        };
        search::hybrid(&self.store, kb_id, query, vector.as_deref(), top_k).await
    }

    async fn parse(&self, file_id: &str, format: Option<&str>) -> Result<Value, KnowledgeError> {
        let (file, bytes) = self.files.get(file_id).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        // A chosen format (`pdf`, `docx`, `md`…) picks the parser by extension, whatever the file is called.
        let name = format.map_or_else(|| file.name.clone(), |f| format!("{}.{f}", file.name));
        let mime = if format.is_some() { String::new() } else { file.mime.clone() };
        let doc = tokio::task::spawn_blocking(move || parse::parse(&name, &mime, &bytes))
            .await
            .map_err(|e| KnowledgeError::Ingest(e.to_string()))??;
        let text = doc.sections.iter().map(|s| s.text.trim()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(
            "

",
        );
        let sections: Vec<Value> =
            doc.sections.iter().map(|section| json!({"heading": section.heading, "text": section.text})).collect();
        let title = if format.is_some() { file.name } else { doc.title };
        Ok(json!({"title": title, "text": text, "sections": sections}))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use il_sdk::services::files::FileStore;
    use il_sdk::services::knowledge::{KnowledgeService, SourceInput};
    use il_sdk::services::models::{ChatRequest, ChatResponse, ChatStream, ModelError, ModelRole, Transcript};
    use il_sdk::testkit::{FakeFiles, FakeGateway};
    use il_store::db::SqliteStore;

    use super::Knowledge;

    #[tokio::test]
    async fn q11_embedding_in_batches_keeps_each_vector_with_its_text() {
        use il_sdk::services::ModelGateway;
        let dir = tempfile::tempdir().unwrap();
        let models = Arc::new(FakeGateway::default());
        let store = Arc::new(SqliteStore::open(dir.path().join("k.sqlite")).unwrap());
        let service = Knowledge::new(store, Arc::new(FakeFiles::default()), models.clone(), None);
        let texts: Vec<String> = (0..300).map(|i| format!("chunk {i} {}", "x".repeat(i % 7))).collect();
        let batched = service.embed_all(&texts).await.unwrap();
        assert_eq!(batched, models.embed(&texts).await.unwrap(), "300 texts span 3 batches, in order");
    }

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

    /// The fake gateway, except that the embedding requests numbered in `hang` never answer.
    struct Stalls {
        inner: FakeGateway,
        calls: std::sync::atomic::AtomicUsize,
        hang: std::ops::Range<usize>,
    }

    #[async_trait::async_trait]
    impl il_sdk::services::ModelGateway for Stalls {
        async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError> {
            self.inner.chat(role, req).await
        }
        async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
            self.inner.chat_stream(role, req).await
        }
        async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
            if self.hang.contains(&self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst)) {
                std::future::pending::<()>().await;
            }
            self.inner.embed(texts).await
        }
        async fn transcribe(&self, file: &il_core::FileRef, bytes: &[u8]) -> Result<Transcript, ModelError> {
            self.inner.transcribe(file, bytes).await
        }
    }

    /// A base with one source, searched while the embedding requests in `hang` stall.
    async fn search_while_stalled(
        hang: std::ops::Range<usize>,
    ) -> (Vec<il_sdk::services::knowledge::SearchHit>, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(dir.path().join("k.sqlite")).unwrap());
        // Request 0 embeds the source; the query's requests come after.
        let models = Arc::new(Stalls { inner: FakeGateway::default(), calls: Default::default(), hang });
        let service = Knowledge::new(store, Arc::new(FakeFiles::default()), models, None);
        let kb = service.create_base("ws_test", "Help").await.unwrap();
        let ids = service
            .add_sources(
                &kb,
                vec![SourceInput::Text { title: "Returns".into(), text: "Return within thirty days.".into() }],
            )
            .await
            .unwrap();
        let hits = tokio::time::timeout(std::time::Duration::from_secs(10), service.search(&kb, "return", 5))
            .await
            .expect("the search doesn't wait on a stalled request")
            .unwrap();
        (hits, ids[0].clone())
    }

    #[tokio::test]
    async fn q28_a_stalled_query_embedding_is_sent_again() {
        let (hits, source) = search_while_stalled(1..2).await;
        assert_eq!(hits[0].source_id, source);
    }

    #[tokio::test]
    async fn q28_search_goes_on_by_keywords_when_embeddings_stall() {
        let (hits, source) = search_while_stalled(1..3).await;
        assert_eq!(hits[0].source_id, source);
    }

    #[tokio::test]
    async fn q28_vector_search_stays_inside_its_base() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(dir.path().join("k.sqlite")).unwrap());
        let service = Knowledge::new(store, Arc::new(FakeFiles::default()), Arc::new(FakeGateway::default()), None);
        let mut bases = Vec::new();
        for (name, text) in
            [("Returns", "You may return an item within thirty days."), ("Shipping", "Orders ship in two days.")]
        {
            let kb = service.create_base("ws_test", name).await.unwrap();
            let ids = service
                .add_sources(&kb, vec![SourceInput::Text { title: name.into(), text: text.into() }])
                .await
                .unwrap();
            bases.push((kb, ids[0].clone()));
        }
        for (kb, source) in &bases {
            // No word matches, so every hit comes from the vector search.
            let hits = service.search(kb, "zzz", 5).await.unwrap();
            assert!(!hits.is_empty() && hits.iter().all(|h| &h.source_id == source), "{hits:?}");
        }
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

    #[tokio::test]
    async fn k03_parse_uses_the_chosen_format_and_returns_text() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(dir.path().join("knowledge.sqlite")).unwrap());
        let files = Arc::new(FakeFiles::default());
        let doc = files
            .put(
                "notes",
                "application/octet-stream",
                "# Refunds

Within 30 days."
                    .into(),
            )
            .await
            .unwrap();
        let service = Knowledge::new(store, files, Arc::new(FakeGateway::default()), None);
        assert!(service.parse(&doc.id, None).await.is_err(), "no extension and an unknown type");
        let parsed = service.parse(&doc.id, Some("md")).await.unwrap();
        assert_eq!(parsed["title"], "notes");
        assert!(parsed["text"].as_str().unwrap().contains("Within 30 days."));
        assert!(!parsed["sections"].as_array().unwrap().is_empty());
    }
}
