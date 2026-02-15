use kc_core::db::open_db;
use kc_core::hashing::blake3_hex_prefixed;
use kc_core::locator::{resolve_locator_strict, LocatorRange, LocatorV1};
use kc_core::object_store::ObjectStore;
use kc_core::snippet::render_snippet_display_only;
use kc_core::types::{CanonicalHash, DocId};

fn setup_doc() -> (rusqlite::Connection, ObjectStore, DocId, CanonicalHash) {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let db = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(root.join("store/objects"));

    let original = b"source bytes";
    let original_hash = store.put_bytes(&db, original, 1).expect("store original");
    let doc_id = DocId(original_hash.0.clone());

    db.execute(
        "INSERT INTO docs (doc_id, original_object_hash, bytes, mime, source_kind, effective_ts_ms, ingested_event_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![doc_id.0, original_hash.0, original.len() as i64, "text/plain", "notes", 1i64, 1i64],
    )
    .expect("insert doc");

    let canonical_text = "[[H1:Title]]\nhello world\n";
    let canonical_hash = CanonicalHash(blake3_hex_prefixed(canonical_text.as_bytes()));
    let canonical_obj = store
        .put_bytes(&db, canonical_text.as_bytes(), 2)
        .expect("store canonical");

    db.execute(
        "INSERT INTO canonical_text (
          doc_id,
          canonical_object_hash,
          canonical_hash,
          extractor_name,
          extractor_version,
          extractor_flags_json,
          normalization_version,
          toolchain_json,
          created_event_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            doc_id.0,
            canonical_obj.0,
            canonical_hash.0,
            "test",
            "1",
            "{}",
            1i64,
            "{}",
            2i64,
        ],
    )
    .expect("insert canonical");

    (db, store, doc_id, canonical_hash)
}

#[test]
fn locator_resolve_strict_success() {
    let (db, store, doc_id, canonical_hash) = setup_doc();
    let locator = LocatorV1 {
        v: 1,
        doc_id,
        canonical_hash,
        range: LocatorRange { start: 13, end: 18 },
        hints: None,
    };

    let got = resolve_locator_strict(&db, &store, &locator).expect("resolve");
    assert_eq!(got, "hello");
}

#[test]
fn locator_hash_mismatch_fails() {
    let (db, store, doc_id, _canonical_hash) = setup_doc();
    let locator = LocatorV1 {
        v: 1,
        doc_id,
        canonical_hash: CanonicalHash(
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ),
        range: LocatorRange { start: 0, end: 5 },
        hints: None,
    };

    let err = resolve_locator_strict(&db, &store, &locator).expect_err("must fail");
    assert_eq!(err.code, "KC_LOCATOR_CANONICAL_HASH_MISMATCH");
}

#[test]
fn locator_range_oob_fails() {
    let (db, store, doc_id, canonical_hash) = setup_doc();
    let locator = LocatorV1 {
        v: 1,
        doc_id,
        canonical_hash,
        range: LocatorRange { start: 0, end: 999 },
        hints: None,
    };

    let err = resolve_locator_strict(&db, &store, &locator).expect_err("must fail");
    assert_eq!(err.code, "KC_LOCATOR_RANGE_OOB");
}

#[test]
fn snippet_display_only_strips_markers() {
    let rendered = render_snippet_display_only("[[PAGE:0001]]\n[[H1:Title]]\nhello\n\n\nworld\n")
        .expect("render");
    assert_eq!(rendered, "hello\n\nworld");
}
