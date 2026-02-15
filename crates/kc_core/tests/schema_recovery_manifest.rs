use jsonschema::validator_for;
use kc_core::recovery::generate_recovery_bundle;
use kc_core::vault::vault_init;

fn recovery_manifest_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/recovery-bundle-manifest/v2",
      "type": "object",
      "required": [
        "schema_version",
        "vault_id",
        "created_at_ms",
        "phrase_checksum",
        "payload_hash"
      ],
      "properties": {
        "schema_version": { "const": 2 },
        "vault_id": { "type": "string", "minLength": 1 },
        "created_at_ms": { "type": "integer" },
        "phrase_checksum": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "payload_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "escrow": {
          "type": "object",
          "required": ["provider", "provider_ref", "key_id", "wrapped_at_ms"],
          "properties": {
            "provider": { "type": "string", "minLength": 1 },
            "provider_ref": { "type": "string", "minLength": 1 },
            "key_id": { "type": "string", "minLength": 1 },
            "wrapped_at_ms": { "type": "integer" }
          },
          "additionalProperties": false
        },
        "escrow_descriptors": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["provider", "provider_ref", "key_id", "wrapped_at_ms"],
            "properties": {
              "provider": { "type": "string", "minLength": 1 },
              "provider_ref": { "type": "string", "minLength": 1 },
              "key_id": { "type": "string", "minLength": 1 },
              "wrapped_at_ms": { "type": "integer" }
            },
            "additionalProperties": false
          }
        }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_recovery_manifest_accepts_generated_payload() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault = vault_init(&root.join("vault"), "demo", 1).expect("vault init");
    let generated =
        generate_recovery_bundle(&vault.vault_id, &root.join("out"), "vault-passphrase", 100)
            .expect("generate recovery");
    let value = serde_json::to_value(generated.manifest).expect("manifest value");
    let schema = validator_for(&recovery_manifest_schema()).expect("compile schema");
    assert!(schema.is_valid(&value));
}

#[test]
fn schema_recovery_manifest_rejects_missing_payload_hash() {
    let schema = validator_for(&recovery_manifest_schema()).expect("compile schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "vault_id": "vault-id",
      "created_at_ms": 100,
      "phrase_checksum": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    assert!(!schema.is_valid(&invalid));
}

#[test]
fn schema_recovery_manifest_rejects_invalid_escrow_descriptor() {
    let schema = validator_for(&recovery_manifest_schema()).expect("compile schema");
    let invalid = serde_json::json!({
      "schema_version": 2,
      "vault_id": "vault-id",
      "created_at_ms": 100,
      "phrase_checksum": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "payload_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "escrow": {
        "provider": "aws",
        "provider_ref": "vault/path/blob.enc",
        "wrapped_at_ms": 100
      }
    });
    assert!(!schema.is_valid(&invalid));
}

#[test]
fn schema_recovery_manifest_accepts_descriptors_array_payload() {
    let schema = validator_for(&recovery_manifest_schema()).expect("compile schema");
    let value = serde_json::json!({
      "schema_version": 2,
      "vault_id": "vault-id",
      "created_at_ms": 300,
      "phrase_checksum": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "payload_hash": "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "escrow_descriptors": [
        {
          "provider": "aws",
          "provider_ref": "secret://vault/aws",
          "key_id": "kms://aws/demo",
          "wrapped_at_ms": 300
        },
        {
          "provider": "gcp",
          "provider_ref": "secret://vault/gcp",
          "key_id": "kms://gcp/demo",
          "wrapped_at_ms": 301
        }
      ]
    });
    assert!(schema.is_valid(&value));
}
