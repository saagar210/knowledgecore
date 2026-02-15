use jsonschema::JSONSchema;
use kc_ask::{AskRequest, RetrievedOnlyAskService};
use kc_core::locator::{LocatorRange, LocatorV1};
use kc_core::types::{CanonicalHash, DocId};
use kc_core::vault::vault_init;

fn sample_locator() -> LocatorV1 {
    LocatorV1 {
        v: 1,
        doc_id: DocId(
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ),
        canonical_hash: CanonicalHash(
            "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        ),
        range: LocatorRange { start: 0, end: 5 },
        hints: None,
    }
}

fn trace_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/trace-log/v1",
      "type": "object",
      "required": ["schema_version", "trace_id", "ts_ms", "vault_id", "question", "retrieval", "model", "answer", "redaction"],
      "properties": {
        "schema_version": { "const": 1 },
        "trace_id": {
          "type": "string",
          "format": "uuid",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },
        "ts_ms": { "type": "integer" },
        "vault_id": {
          "type": "string",
          "format": "uuid",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },
        "question": { "type": "string" },
        "retrieval": { "type": "object" },
        "model": { "type": "object" },
        "answer": { "type": "object" },
        "redaction": { "type": "object" }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_trace_accepts_written_trace() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "ask", 1).expect("vault init");
    let service = RetrievedOnlyAskService::default();

    let out = service
        .finalize_answer(
            &AskRequest {
                vault_path: root,
                question: "What happened?".to_string(),
                now_ms: 2,
            },
            "answer".to_string(),
            vec![(0, vec![sample_locator()])],
        )
        .expect("finalize");

    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.trace_path).expect("read trace"))
            .expect("parse trace");
    let schema = JSONSchema::compile(&trace_schema()).expect("compile trace schema");
    assert!(schema.is_valid(&value));
}

#[test]
fn schema_trace_rejects_non_uuid_ids() {
    let schema = JSONSchema::compile(&trace_schema()).expect("compile trace schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "trace_id": "not-a-uuid",
      "ts_ms": 1,
      "vault_id": "not-a-uuid",
      "question": "q",
      "retrieval": {},
      "model": {},
      "answer": {},
      "redaction": {}
    });
    assert!(!schema.is_valid(&invalid));
}
