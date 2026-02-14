use kc_core::db::open_db;
use kc_core::events::append_event;
use kc_core::hashing::blake3_hex_prefixed;
use kc_core::ingest::ingest_bytes;
use kc_core::lineage::query_lineage;
use kc_core::object_store::ObjectStore;
use kc_core::vault::vault_init;
use rusqlite::params;

fn fixed_hash(c: char) -> String {
    format!("blake3:{}", c.to_string().repeat(64))
}

#[test]
fn lineage_query_is_deterministic_and_sorted() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    vault_init(&vault_root, "demo", 1).expect("vault init");

    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(vault_root.join("store/objects"));
    let ingested = ingest_bytes(
        &conn,
        &store,
        b"hello lineage",
        "text/plain",
        "notes",
        1,
        Some("/tmp/z-source.txt"),
        10,
    )
    .expect("ingest");

    conn.execute(
        "INSERT INTO doc_sources(doc_id, source_path) VALUES (?1, ?2)",
        params![ingested.doc_id.0.clone(), "/tmp/a-source.txt"],
    )
    .expect("insert extra source");

    let canonical_event = append_event(
        &conn,
        11,
        "canonical.persist",
        &serde_json::json!({ "doc_id": ingested.doc_id.0 }),
    )
    .expect("append canonical event");
    let canonical_bytes = b"canonical";
    let canonical_hash = blake3_hex_prefixed(canonical_bytes);
    let canonical_object = store
        .put_bytes(&conn, canonical_bytes, canonical_event.event_id)
        .expect("put canonical object");

    conn.execute(
        "INSERT INTO canonical_text (
            doc_id, canonical_object_hash, canonical_hash, extractor_name, extractor_version,
            extractor_flags_json, normalization_version, toolchain_json, created_event_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            ingested.doc_id.0.clone(),
            canonical_object.0,
            canonical_hash,
            "unit",
            "1.0.0",
            "{}",
            1i64,
            "{}",
            canonical_event.event_id
        ],
    )
    .expect("insert canonical row");

    conn.execute(
        "INSERT INTO chunks(chunk_id, doc_id, ordinal, start_char, end_char, chunking_config_hash, source_kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "chunk-b",
            ingested.doc_id.0.clone(),
            1i64,
            10i64,
            20i64,
            fixed_hash('b'),
            "notes"
        ],
    )
    .expect("insert chunk b");
    conn.execute(
        "INSERT INTO chunks(chunk_id, doc_id, ordinal, start_char, end_char, chunking_config_hash, source_kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            "chunk-a",
            ingested.doc_id.0.clone(),
            0i64,
            0i64,
            10i64,
            fixed_hash('a'),
            "notes"
        ],
    )
    .expect("insert chunk a");

    let res_a = query_lineage(&conn, &ingested.doc_id.0, 2, 99).expect("lineage a");
    let res_b = query_lineage(&conn, &ingested.doc_id.0, 2, 99).expect("lineage b");

    assert_eq!(res_a, res_b);
    assert_eq!(res_a.schema_version, 1);
    assert_eq!(res_a.seed_doc_id, ingested.doc_id.0);

    let node_keys: Vec<(String, String)> = res_a
        .nodes
        .iter()
        .map(|n| (n.kind.clone(), n.node_id.clone()))
        .collect();
    let mut node_keys_sorted = node_keys.clone();
    node_keys_sorted.sort();
    assert_eq!(node_keys, node_keys_sorted);

    let edge_keys: Vec<(String, String, String)> = res_a
        .edges
        .iter()
        .map(|e| {
            (
                e.from_node_id.clone(),
                e.to_node_id.clone(),
                e.relation.clone(),
            )
        })
        .collect();
    let mut edge_keys_sorted = edge_keys.clone();
    edge_keys_sorted.sort();
    assert_eq!(edge_keys, edge_keys_sorted);

    assert!(res_a
        .edges
        .iter()
        .any(|e| e.relation == "contains_chunk" && e.to_node_id == "chunk:chunk-a"));
    assert!(res_a
        .edges
        .iter()
        .any(|e| e.relation == "contains_chunk" && e.to_node_id == "chunk:chunk-b"));
}

#[test]
fn lineage_query_rejects_invalid_depth() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");

    let err = query_lineage(&conn, "missing", 0, 1).expect_err("invalid depth must fail");
    assert_eq!(err.code, "KC_LINEAGE_INVALID_DEPTH");
}

#[test]
fn lineage_query_reports_missing_seed_doc() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");

    let err = query_lineage(&conn, "does-not-exist", 1, 1).expect_err("missing doc must fail");
    assert_eq!(err.code, "KC_LINEAGE_DOC_NOT_FOUND");
}
