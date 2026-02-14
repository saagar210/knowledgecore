use kc_ask::{AskRequest, AskService, RetrievedOnlyAskService};
use kc_core::canonical::persist_canonical_text;
use kc_core::db::open_db;
use kc_core::hashing::blake3_hex_prefixed;
use kc_core::ingest::ingest_bytes;
use kc_core::locator::{LocatorRange, LocatorV1};
use kc_core::object_store::ObjectStore;
use kc_core::services::CanonicalTextArtifact;
use kc_core::types::{CanonicalHash, DocId};
use kc_core::vault::vault_init;

fn sample_locator(v: i64) -> LocatorV1 {
    LocatorV1 {
        v,
        doc_id: DocId("blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
        canonical_hash: CanonicalHash("blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string()),
        range: LocatorRange { start: 0, end: 5 },
        hints: None,
    }
}

#[test]
fn ask_missing_citations_hard_fails() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "ask", 1).expect("vault init");

    let service = RetrievedOnlyAskService::default();
    let req = AskRequest {
        vault_path: root,
        question: "What happened?".to_string(),
        now_ms: 2,
    };

    let err = service
        .finalize_answer(&req, "answer".to_string(), vec![])
        .expect_err("must fail");
    assert_eq!(err.code, "KC_ASK_MISSING_CITATIONS");
}

#[test]
fn ask_invalid_citations_hard_fails() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "ask", 1).expect("vault init");

    let service = RetrievedOnlyAskService::default();
    let req = AskRequest {
        vault_path: root,
        question: "What happened?".to_string(),
        now_ms: 2,
    };

    let err = service
        .finalize_answer(&req, "answer".to_string(), vec![(0, vec![sample_locator(2)])])
        .expect_err("must fail");
    assert_eq!(err.code, "KC_ASK_INVALID_CITATIONS");
}

#[test]
fn ask_writes_trace_for_valid_citations() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "ask", 1).expect("vault init");

    let service = RetrievedOnlyAskService::default();
    let req = AskRequest {
        vault_path: root,
        question: "What happened?".to_string(),
        now_ms: 2,
    };

    let out = service
        .finalize_answer(&req, "answer".to_string(), vec![(0, vec![sample_locator(1)])])
        .expect("success");

    assert!(out.trace_path.exists());
    let trace: serde_json::Value = serde_json::from_slice(&std::fs::read(out.trace_path).expect("read trace"))
        .expect("parse trace");
    assert_eq!(trace.get("schema_version").and_then(|v| v.as_i64()), Some(1));
}

#[test]
fn ask_runs_retrieved_only_end_to_end() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "ask", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
    let store = ObjectStore::new(root.join("store/objects"));

    let ingested = ingest_bytes(
        &conn,
        &store,
        b"raw bytes",
        "text/plain",
        "notes",
        1,
        None,
        1,
    )
    .expect("ingest");

    let canonical_text = b"Evidence paragraph for answer.\n".to_vec();
    let canonical_hash = blake3_hex_prefixed(&canonical_text);
    let artifact = CanonicalTextArtifact {
        doc_id: ingested.doc_id.clone(),
        canonical_bytes: canonical_text.clone(),
        canonical_hash: CanonicalHash(canonical_hash.clone()),
        canonical_object_hash: kc_core::types::ObjectHash(canonical_hash),
        extractor_name: "test".to_string(),
        extractor_version: "1".to_string(),
        extractor_flags_json: "{}".to_string(),
        normalization_version: 1,
        toolchain_json: "{}".to_string(),
    };
    persist_canonical_text(&conn, &store, &artifact, 1).expect("persist canonical");

    let service = RetrievedOnlyAskService::default();
    let out = service
        .ask(AskRequest {
            vault_path: root.clone(),
            question: "What is the evidence?".to_string(),
            now_ms: 2,
        })
        .expect("ask");

    assert!(!out.answer_text.is_empty());
    assert!(!out.citations.is_empty());
    assert!(out.trace_path.exists());
}
