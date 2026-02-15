use jsonschema::validator_for;

fn lineage_lock_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-lock/v1",
      "type": "object",
      "required": ["doc_id", "owner", "token", "acquired_at_ms", "expires_at_ms"],
      "properties": {
        "doc_id": { "type": "string", "minLength": 1 },
        "owner": { "type": "string", "minLength": 1 },
        "token": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "acquired_at_ms": { "type": "integer" },
        "expires_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

fn lineage_lock_status_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-lock-status/v1",
      "type": "object",
      "required": ["doc_id", "held", "owner", "acquired_at_ms", "expires_at_ms", "expired"],
      "properties": {
        "doc_id": { "type": "string", "minLength": 1 },
        "held": { "type": "boolean" },
        "owner": { "type": ["string", "null"] },
        "acquired_at_ms": { "type": ["integer", "null"] },
        "expires_at_ms": { "type": ["integer", "null"] },
        "expired": { "type": "boolean" }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_lineage_lock_accepts_valid_payload() {
    let schema = validator_for(&lineage_lock_schema()).expect("compile lineage lock schema");
    let payload = serde_json::json!({
      "doc_id": "doc-1",
      "owner": "tester",
      "token": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "acquired_at_ms": 100,
      "expires_at_ms": 200
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_lock_status_accepts_valid_payload() {
    let schema =
        validator_for(&lineage_lock_status_schema()).expect("compile lineage lock status schema");
    let payload = serde_json::json!({
      "doc_id": "doc-1",
      "held": true,
      "owner": "tester",
      "acquired_at_ms": 100,
      "expires_at_ms": 200,
      "expired": false
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_lock_rejects_missing_token() {
    let schema = validator_for(&lineage_lock_schema()).expect("compile lineage lock schema");
    let invalid = serde_json::json!({
      "doc_id": "doc-1",
      "owner": "tester",
      "acquired_at_ms": 100,
      "expires_at_ms": 200
    });
    assert!(!schema.is_valid(&invalid));
}
