//! A saved base's stored sources, for exporting a flow that uses it (Q36). The copy carries each
//! source's parsed text, so loading it elsewhere needs no crawling or file parsing, only embedding.

use il_sdk::services::StoreError;
use il_sdk::services::knowledge::{BaseCopy, SourceText};
use il_store::db::SqliteStore;
use rusqlite::{OptionalExtension, params};

use super::db;

pub(super) async fn export_base(
    store: &SqliteStore,
    workspace_id: &str,
    kb_id: &str,
) -> Result<Option<BaseCopy>, StoreError> {
    let (workspace, id) = (workspace_id.to_owned(), kb_id.to_owned());
    store
        .with_conn(move |conn| {
            let name: Option<String> = conn
                .query_row("SELECT name FROM kb_bases WHERE id=?1 AND workspace_id=?2", params![id, workspace], |r| {
                    r.get(0)
                })
                .optional()
                .map_err(db)?;
            let Some(name) = name else { return Ok(None) };
            let mut stmt = conn
                .prepare("SELECT title, url, role, content FROM kb_sources WHERE kb_id=?1 ORDER BY created_at, id")
                .map_err(db)?;
            let sources = stmt
                .query_map(params![id], |r| {
                    Ok(SourceText { title: r.get(0)?, url: r.get(1)?, role: r.get(2)?, text: r.get(3)? })
                })
                .map_err(db)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db)?;
            Ok(Some(BaseCopy { name, sources }))
        })
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use il_sdk::services::knowledge::{KnowledgeService, SourceInput};
    use il_sdk::testkit::{FakeFiles, FakeGateway};
    use il_store::db::SqliteStore;

    use crate::ingest::Knowledge;

    #[tokio::test]
    async fn q36_a_saved_base_exports_its_sources_text_for_its_workspace_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(dir.path().join("k.sqlite")).unwrap());
        let service = Knowledge::new(store, Arc::new(FakeFiles::default()), Arc::new(FakeGateway::default()), None);
        let kb = service.create_base("ws_a", "Policies").await.unwrap();
        service
            .add_sources(
                &kb,
                vec![
                    SourceInput::Text { title: "Returns".into(), text: "Return items within thirty days.".into() },
                    SourceInput::InstructionsText { title: "Tone".into(), text: "Be brief.".into() },
                ],
            )
            .await
            .unwrap();
        let copy = service.export_base("ws_a", &kb).await.unwrap().unwrap();
        assert_eq!(copy.name, "Policies");
        let sources: Vec<_> =
            copy.sources.iter().map(|s| (s.title.as_str(), s.role.as_str(), s.text.as_str())).collect();
        assert_eq!(
            sources,
            [("Returns", "knowledge", "Return items within thirty days."), ("Tone", "instructions", "Be brief.")]
        );
        assert!(service.export_base("ws_b", &kb).await.unwrap().is_none(), "another workspace's base");
    }
}
