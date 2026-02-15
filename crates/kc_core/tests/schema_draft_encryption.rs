use jsonschema::validator_for;

fn draft_encryption_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/draft/encryption-metadata/v1",
      "type": "object",
      "required": [
        "schema_version",
        "status",
        "activation_phase",
        "cipher_suite",
        "kdf",
        "key_reference"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "status": { "const": "draft" },
        "activation_phase": { "const": "M" },
        "cipher_suite": { "type": "string" },
        "kdf": {
          "type": "object",
          "required": ["algorithm", "memory_kib", "iterations", "parallelism", "salt_id"],
          "properties": {
            "algorithm": { "type": "string" },
            "memory_kib": { "type": "integer", "minimum": 1 },
            "iterations": { "type": "integer", "minimum": 1 },
            "parallelism": { "type": "integer", "minimum": 1 },
            "salt_id": { "type": "string" }
          },
          "additionalProperties": false
        },
        "key_reference": { "type": "string" }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_draft_encryption_accepts_representative_payload() {
    let schema =
        validator_for(&draft_encryption_schema()).expect("compile draft encryption schema");
    let value = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "M",
      "cipher_suite": "xchacha20poly1305",
      "kdf": {
        "algorithm": "argon2id",
        "memory_kib": 65536,
        "iterations": 3,
        "parallelism": 1,
        "salt_id": "draft-salt-id-v1"
      },
      "key_reference": "draft-key-reference-v1"
    });

    assert!(schema.is_valid(&value));
}

#[test]
fn schema_draft_encryption_rejects_missing_kdf() {
    let schema =
        validator_for(&draft_encryption_schema()).expect("compile draft encryption schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "M",
      "cipher_suite": "xchacha20poly1305",
      "key_reference": "draft-key-reference-v1"
    });

    assert!(!schema.is_valid(&invalid));
}
