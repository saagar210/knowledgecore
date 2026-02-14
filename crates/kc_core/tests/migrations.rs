use kc_core::db::{open_db, schema_version};

#[test]
fn migrations_apply_schema_v1() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db_path = temp.path().join("db/knowledge.sqlite");

    let conn = open_db(&db_path).expect("open db");
    let version = schema_version(&conn).expect("schema version");
    assert_eq!(version, 1);

    let names: Vec<String> = ["objects", "docs", "doc_sources", "canonical_text", "chunks", "events"]
        .iter()
        .map(|table| {
            conn.query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get::<_, String>(0),
            )
            .expect("table must exist")
        })
        .collect();

    assert_eq!(names.len(), 6);
}
