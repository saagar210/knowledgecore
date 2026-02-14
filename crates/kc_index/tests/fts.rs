use kc_index::fts::{init_fts, query, rebuild_rows, FtsRow};

#[test]
fn fts_rebuild_order_is_deterministic() {
    let conn = rusqlite::Connection::open_in_memory().expect("memory db");
    init_fts(&conn).expect("init");

    rebuild_rows(
        &conn,
        &[
            FtsRow {
                chunk_id: "c3".to_string(),
                doc_id: "d2".to_string(),
                ordinal: 1,
                content: "gamma".to_string(),
            },
            FtsRow {
                chunk_id: "c1".to_string(),
                doc_id: "d1".to_string(),
                ordinal: 0,
                content: "alpha".to_string(),
            },
            FtsRow {
                chunk_id: "c2".to_string(),
                doc_id: "d1".to_string(),
                ordinal: 1,
                content: "beta".to_string(),
            },
        ],
    )
    .expect("rebuild");

    let ordered: Vec<String> = conn
        .prepare("SELECT chunk_id FROM chunks_fts ORDER BY rowid")
        .expect("prepare")
        .query_map([], |row| row.get(0))
        .expect("query")
        .map(|x| x.expect("row"))
        .collect();

    assert_eq!(ordered, vec!["c1", "c2", "c3"]);
}

#[test]
fn fts_query_returns_candidates() {
    let conn = rusqlite::Connection::open_in_memory().expect("memory db");
    rebuild_rows(
        &conn,
        &[
            FtsRow {
                chunk_id: "c1".to_string(),
                doc_id: "d1".to_string(),
                ordinal: 0,
                content: "hello world".to_string(),
            },
            FtsRow {
                chunk_id: "c2".to_string(),
                doc_id: "d2".to_string(),
                ordinal: 0,
                content: "something else".to_string(),
            },
        ],
    )
    .expect("rebuild");

    let hits = query(&conn, "hello", 10).expect("query");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk_id.0, "c1");
}
