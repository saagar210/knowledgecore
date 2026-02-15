use jsonschema::validator_for;
use kc_core::vault::vault_init;

fn vault_json_v3_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/vault-json/v3",
      "type": "object",
      "required": [
        "schema_version",
        "vault_id",
        "vault_slug",
        "created_at_ms",
        "db",
        "defaults",
        "toolchain",
        "encryption",
        "db_encryption"
      ],
      "properties": {
        "schema_version": { "const": 3 },
        "vault_id": { "type": "string", "format": "uuid" },
        "vault_slug": { "type": "string" },
        "created_at_ms": { "type": "integer" },
        "db": {
          "type": "object",
          "required": ["relative_path"],
          "properties": {
            "relative_path": { "type": "string" }
          },
          "additionalProperties": false
        },
        "defaults": { "type": "object" },
        "toolchain": { "type": "object" },
        "encryption": {
          "type": "object",
          "required": ["enabled", "mode", "kdf"],
          "properties": {
            "enabled": { "type": "boolean" },
            "mode": { "const": "object_store_xchacha20poly1305" },
            "kdf": {
              "type": "object",
              "required": ["algorithm", "memory_kib", "iterations", "parallelism", "salt_id"],
              "properties": {
                "algorithm": { "const": "argon2id" },
                "memory_kib": { "type": "integer", "minimum": 1 },
                "iterations": { "type": "integer", "minimum": 1 },
                "parallelism": { "type": "integer", "minimum": 1 },
                "salt_id": { "type": "string", "minLength": 8 }
              },
              "additionalProperties": false
            },
            "key_reference": { "type": ["string", "null"] }
          },
          "additionalProperties": false
        },
        "db_encryption": {
          "type": "object",
          "required": ["enabled", "mode", "kdf"],
          "properties": {
            "enabled": { "type": "boolean" },
            "mode": { "const": "sqlcipher_v4" },
            "kdf": {
              "type": "object",
              "required": ["algorithm"],
              "properties": {
                "algorithm": { "const": "pbkdf2_hmac_sha512" }
              },
              "additionalProperties": false
            },
            "key_reference": { "type": ["string", "null"] }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_vault_json_accepts_generated_v3_payload() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let created = vault_init(&root, "demo", 123).expect("vault init");
    let value = serde_json::to_value(created).expect("serialize vault json");
    let schema = validator_for(&vault_json_v3_schema()).expect("compile vault schema");
    assert!(schema.is_valid(&value));
}

#[test]
fn schema_vault_json_rejects_missing_db_encryption_block() {
    let invalid = serde_json::json!({
      "schema_version": 3,
      "vault_id": "2f9709fe-dda6-41d6-93c6-f1a0d5f9f3fd",
      "vault_slug": "demo",
      "created_at_ms": 1,
      "db": { "relative_path": "db/knowledge.sqlite" },
      "defaults": {},
      "toolchain": {},
      "encryption": {
        "enabled": false,
        "mode": "object_store_xchacha20poly1305",
        "kdf": {
          "algorithm": "argon2id",
          "memory_kib": 65536,
          "iterations": 3,
          "parallelism": 1,
          "salt_id": "vault-kdf-salt-v1"
        },
        "key_reference": null
      }
    });
    let schema = validator_for(&vault_json_v3_schema()).expect("compile vault schema");
    assert!(!schema.is_valid(&invalid));
}
