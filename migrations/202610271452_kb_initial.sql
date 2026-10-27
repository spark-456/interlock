CREATE TABLE kb_bases (
  id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, name TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX kb_bases_workspace ON kb_bases(workspace_id);

CREATE TABLE kb_sources (
  id TEXT PRIMARY KEY, kb_id TEXT NOT NULL REFERENCES kb_bases(id) ON DELETE CASCADE,
  title TEXT NOT NULL, url TEXT, role TEXT NOT NULL, content TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX kb_sources_base ON kb_sources(kb_id);

CREATE TABLE kb_chunks (
  id INTEGER PRIMARY KEY, kb_id TEXT NOT NULL REFERENCES kb_bases(id) ON DELETE CASCADE,
  source_id TEXT NOT NULL REFERENCES kb_sources(id) ON DELETE CASCADE,
  title TEXT NOT NULL, section TEXT, date TEXT NOT NULL, text TEXT NOT NULL
);
CREATE INDEX kb_chunks_base ON kb_chunks(kb_id);

CREATE VIRTUAL TABLE kb_chunks_fts USING fts5(text, content='kb_chunks', content_rowid='id');
CREATE TRIGGER kb_chunks_insert AFTER INSERT ON kb_chunks BEGIN
  INSERT INTO kb_chunks_fts(rowid, text) VALUES (new.id, new.text);
END;
CREATE TRIGGER kb_chunks_delete AFTER DELETE ON kb_chunks BEGIN
  INSERT INTO kb_chunks_fts(kb_chunks_fts, rowid, text) VALUES ('delete', old.id, old.text);
END;

-- vec0 requires a fixed dimension. The ingestion service creates kb_chunks_vec on
-- the first embedding and records its dimension here, then rejects mismatches.
CREATE TABLE kb_vec_meta (id INTEGER PRIMARY KEY CHECK(id = 1), dimensions INTEGER NOT NULL);
