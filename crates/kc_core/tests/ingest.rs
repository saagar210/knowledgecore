use kc_core::db::open_db;
use kc_core::ingest::{ingest_bytes, IngestBytesReq};
use kc_core::object_store::ObjectStore;

#[test]
fn ingest_is_idempotent_and_persists_doc_source() {
    let temp = tempfile::tempdir().expect("tempdir");
    let conn = open_db(&temp.path().join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(temp.path().join("store/objects"));

    let input = b"same ingest payload";
    let first = ingest_bytes(
        &conn,
        &store,
        IngestBytesReq {
            bytes: input,
            mime: "text/plain",
            source_kind: "notes",
            effective_ts_ms: 100,
            source_path: Some("/tmp/a.txt"),
            now_ms: 200,
        },
    )
    .expect("first ingest");

    let second = ingest_bytes(
        &conn,
        &store,
        IngestBytesReq {
            bytes: input,
            mime: "text/plain",
            source_kind: "notes",
            effective_ts_ms: 100,
            source_path: Some("/tmp/a.txt"),
            now_ms: 201,
        },
    )
    .expect("second ingest");

    assert_eq!(first.doc_id, second.doc_id);

    let doc_id = first.doc_id.0.clone();
    let docs_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM docs WHERE doc_id=?1",
            [&doc_id],
            |r| r.get(0),
        )
        .expect("docs count");
    assert_eq!(docs_count, 1);

    let src_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM doc_sources WHERE doc_id=?1 AND source_path=?2",
            [&doc_id, "/tmp/a.txt"],
            |r| r.get(0),
        )
        .expect("doc source count");
    assert_eq!(src_count, 1);
}
