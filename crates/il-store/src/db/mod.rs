//! Async SQLite wrapper (WAL mode) and the migrations runner.
//!
//! Owned by task B01.

#[cfg(test)]
mod tests {
    /// Proves the bundled SQLite builds on this platform and has FTS5 (sqlite-vec comes with B01).
    #[test]
    fn bundled_sqlite_has_fts5() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE VIRTUAL TABLE t USING fts5(body); INSERT INTO t VALUES ('larkspur support');")
            .unwrap();
        let n: i64 = conn.query_row("SELECT count(*) FROM t WHERE t MATCH 'larkspur'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
}
