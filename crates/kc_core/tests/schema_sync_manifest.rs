use jsonschema::JSONSchema;

fn sync_head_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/sync-head/v1",
      "type": "object",
      "required": ["schema_version", "snapshot_id", "manifest_hash", "created_at_ms"],
      "properties": {
        "schema_version": { "const": 1 },
        "snapshot_id": { "type": "string" },
        "manifest_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "created_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

fn sync_conflict_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/sync-conflict-artifact/v1",
      "type": "object",
      "required": [
        "schema_version",
        "kind",
        "vault_id",
        "now_ms",
        "local_manifest_hash",
        "remote_head_snapshot_id",
        "remote_head_manifest_hash",
        "seen_remote_snapshot_id"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "kind": { "type": "string", "const": "sync_conflict" },
        "vault_id": { "type": "string" },
        "now_ms": { "type": "integer" },
        "local_manifest_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "remote_head_snapshot_id": { "type": ["string", "null"] },
        "remote_head_manifest_hash": { "type": ["string", "null"] },
        "seen_remote_snapshot_id": { "type": ["string", "null"] }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_sync_head_accepts_valid_payload() {
    let schema = JSONSchema::compile(&sync_head_schema()).expect("compile sync head schema");
    let payload = serde_json::json!({
      "schema_version": 1,
      "snapshot_id": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "manifest_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "created_at_ms": 100
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_sync_conflict_rejects_missing_kind() {
    let schema = JSONSchema::compile(&sync_conflict_schema()).expect("compile sync conflict schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "vault_id": "vault-id",
      "now_ms": 100,
      "local_manifest_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "remote_head_snapshot_id": null,
      "remote_head_manifest_hash": null,
      "seen_remote_snapshot_id": null
    });
    assert!(!schema.is_valid(&invalid));
}
