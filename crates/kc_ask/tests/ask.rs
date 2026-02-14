use kc_ask::{AskRequest, RetrievedOnlyAskService};
use kc_core::locator::{LocatorRange, LocatorV1};
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
