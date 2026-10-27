//! Hybrid sqlite-vec and FTS5 search fused by reciprocal rank.

use std::collections::BTreeMap;

use il_sdk::services::{
    StoreError,
    knowledge::{KnowledgeError, SearchHit},
};
use il_store::db::SqliteStore;
use rusqlite::{OptionalExtension, Row, params};

struct Candidate {
    id: i64,
    hit: SearchHit,
}

fn candidate(row: &Row<'_>) -> rusqlite::Result<Candidate> {
    Ok(Candidate {
        id: row.get(0)?,
        hit: SearchHit {
            source_id: row.get(1)?,
            title: row.get(2)?,
            section: row.get(3)?,
            url: row.get(4)?,
            text: row.get(5)?,
            score: 0.0,
        },
    })
}

/// Returns top results for one base, normalized to scores between zero and one.
pub async fn hybrid(
    store: &SqliteStore,
    kb_id: &str,
    query: &str,
    vector: &[f32],
    top_k: usize,
) -> Result<Vec<SearchHit>, KnowledgeError> {
    let kb_id = kb_id.to_owned();
    let vector = serde_json::to_string(vector).map_err(|e| KnowledgeError::Other(e.to_string()))?;
    let terms = query
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .take(12)
        .map(|part| format!("\"{part}\""))
        .collect::<Vec<_>>()
        .join(" OR ");
    store
        .with_conn(move |conn| {
            let dimension: Option<usize> = conn
                .query_row("SELECT dimensions FROM kb_vec_meta WHERE id=1", [], |row| row.get(0))
                .optional()
                .map_err(db)?;
            if dimension.is_none() {
                return Ok(Vec::new());
            }
            let limit = top_k.clamp(1, 50) as i64;
            let mut fused = BTreeMap::<i64, (SearchHit, f32)>::new();
            let mut vec_stmt = conn
                .prepare(
                    "SELECT c.id,c.source_id,c.title,c.section,s.url,c.text
             FROM kb_chunks_vec v JOIN kb_chunks c ON c.id=v.rowid
             JOIN kb_sources s ON s.id=c.source_id WHERE c.kb_id=?2
             ORDER BY vec_distance_cosine(v.embedding,vec_f32(?1)) LIMIT ?3",
                )
                .map_err(db)?;
            let vectors = vec_stmt.query_map(params![vector, kb_id, limit * 4], candidate).map_err(db)?;
            for (rank, row) in vectors.enumerate() {
                let row = row.map_err(db)?;
                fused.insert(row.id, (row.hit, 1.0 / (60.0 + rank as f32 + 1.0)));
            }
            if !terms.is_empty() {
                let mut fts_stmt = conn
                    .prepare(
                        "SELECT c.id,c.source_id,c.title,c.section,s.url,c.text
                 FROM kb_chunks_fts JOIN kb_chunks c ON c.id=kb_chunks_fts.rowid
                 JOIN kb_sources s ON s.id=c.source_id
                 WHERE kb_chunks_fts MATCH ?1 AND c.kb_id=?2 ORDER BY bm25(kb_chunks_fts) LIMIT ?3",
                    )
                    .map_err(db)?;
                let matches = fts_stmt.query_map(params![terms, kb_id, limit * 4], candidate).map_err(db)?;
                for (rank, row) in matches.enumerate() {
                    let row = row.map_err(db)?;
                    let score = 1.0 / (60.0 + rank as f32 + 1.0);
                    fused.entry(row.id).and_modify(|(_, total)| *total += score).or_insert((row.hit, score));
                }
            }
            let mut ranked: Vec<_> = fused.into_values().collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            let max = ranked.first().map(|(_, score)| *score).unwrap_or(1.0);
            Ok(ranked
                .into_iter()
                .take(limit as usize)
                .map(|(mut hit, score)| {
                    hit.score = score / max;
                    hit
                })
                .collect())
        })
        .await
        .map_err(|e| KnowledgeError::Other(e.to_string()))
}

fn db(error: impl std::fmt::Display) -> StoreError {
    StoreError::Backend(error.to_string())
}
