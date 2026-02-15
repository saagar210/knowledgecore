use jsonschema::validator_for;

fn draft_sync_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/draft/sync-manifest/v1",
      "type": "object",
      "required": [
        "schema_version",
        "status",
        "activation_phase",
        "vault_id",
        "snapshot_id",
        "created_at_ms",
        "objects_hash",
        "db_hash",
        "conflicts"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "status": { "const": "draft" },
        "activation_phase": { "const": "N2" },
        "vault_id": { "type": "string" },
        "snapshot_id": { "type": "string" },
        "created_at_ms": { "type": "integer" },
        "objects_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "db_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "conflicts": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["path", "local_hash", "remote_hash", "resolution_strategy"],
            "properties": {
              "path": { "type": "string" },
              "local_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
              "remote_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
              "resolution_strategy": { "type": "string" }
            },
            "additionalProperties": false
          }
        }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_draft_sync_accepts_representative_payload() {
    let schema = validator_for(&draft_sync_schema()).expect("compile draft sync schema");
    let value = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "N2",
      "vault_id": "draft-vault-id",
      "snapshot_id": "draft-snapshot-id",
      "created_at_ms": 0,
      "objects_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
      "db_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
      "conflicts": [
        {
          "path": "docs/note.md",
          "local_hash": "blake3:1111111111111111111111111111111111111111111111111111111111111111",
          "remote_hash": "blake3:2222222222222222222222222222222222222222222222222222222222222222",
          "resolution_strategy": "emit_conflict_artifact"
        }
      ]
    });

    assert!(schema.is_valid(&value));
}

#[test]
fn schema_draft_sync_rejects_bad_hash_shape() {
    let schema = validator_for(&draft_sync_schema()).expect("compile draft sync schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "N2",
      "vault_id": "draft-vault-id",
      "snapshot_id": "draft-snapshot-id",
      "created_at_ms": 0,
      "objects_hash": "bad",
      "db_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
      "conflicts": []
    });

    assert!(!schema.is_valid(&invalid));
}
