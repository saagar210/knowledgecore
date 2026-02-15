use jsonschema::JSONSchema;
use kc_core::db::open_db;
use kc_core::export::{export_bundle, ExportOptions};
use kc_core::vault::vault_init;

fn export_manifest_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/export-manifest/v1",
      "type": "object",
      "required": ["manifest_version", "vault_id", "schema_versions", "encryption", "db_encryption", "recovery_escrow", "packaging", "chunking_config_hash", "db", "objects"],
      "properties": {
        "manifest_version": { "const": 1 },
        "vault_id": {
          "type": "string",
          "format": "uuid",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },
        "schema_versions": { "type": "object" },
        "encryption": {
          "type": "object",
          "required": ["enabled", "mode", "kdf"],
          "properties": {
            "enabled": { "type": "boolean" },
            "mode": { "type": "string" },
            "key_reference": { "type": ["string", "null"] },
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
            }
          },
          "additionalProperties": false
        },
        "db_encryption": {
          "type": "object",
          "required": ["enabled", "mode", "kdf"],
          "properties": {
            "enabled": { "type": "boolean" },
            "mode": { "type": "string" },
            "key_reference": { "type": ["string", "null"] },
            "kdf": {
              "type": "object",
              "required": ["algorithm"],
              "properties": {
                "algorithm": { "type": "string" }
              },
              "additionalProperties": false
            }
          },
          "additionalProperties": false
        },
        "recovery_escrow": {
          "type": "object",
          "required": ["enabled", "provider", "providers", "updated_at_ms", "descriptor", "escrow_descriptors"],
          "properties": {
            "enabled": { "type": "boolean" },
            "provider": { "type": "string", "minLength": 1 },
            "providers": {
              "type": "array",
              "items": { "type": "string", "minLength": 1 }
            },
            "updated_at_ms": { "type": ["integer", "null"] },
            "descriptor": {
              "type": ["object", "null"],
              "additionalProperties": true
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
        },
        "toolchain_registry": { "type": "object" },
        "packaging": {
          "type": "object",
          "required": ["format", "zip_policy"],
          "properties": {
            "format": { "type": "string", "enum": ["folder", "zip"] },
            "zip_policy": {
              "type": "object",
              "required": ["compression", "mtime", "file_mode"],
              "properties": {
                "compression": { "type": "string", "const": "stored" },
                "mtime": { "type": "string", "const": "1980-01-01T00:00:00Z" },
                "file_mode": { "type": "string", "const": "0644" }
              },
              "additionalProperties": false
            }
          },
          "additionalProperties": false
        },
        "chunking_config_id": { "type": "string" },
        "chunking_config_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "embedding": { "type": "object" },
        "db": {
          "type": "object",
          "required": ["relative_path", "hash"],
          "properties": {
            "relative_path": { "type": "string" },
            "hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" }
          },
          "additionalProperties": false
        },
        "objects": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["relative_path", "hash", "storage_hash", "encrypted", "bytes"],
            "properties": {
              "relative_path": { "type": "string" },
              "hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
              "storage_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
              "encrypted": { "type": "boolean" },
              "bytes": { "type": "integer", "minimum": 0 }
            },
            "additionalProperties": false
          }
        },
        "indexes": { "type": "object" }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_export_manifest_accepts_generated_manifest() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let export_root = root.join("exports");
    vault_init(&vault_root, "demo", 1000).expect("vault init");
    let _ = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");

    let bundle = export_bundle(
        &vault_root,
        &export_root,
        &ExportOptions {
            include_vectors: false,
            as_zip: false,
        },
        123,
    )
    .expect("export bundle");

    let value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(bundle.join("manifest.json")).expect("read manifest"),
    )
    .expect("parse manifest");

    let schema = JSONSchema::compile(&export_manifest_schema()).expect("compile schema");
    assert!(schema.is_valid(&value));
}

#[test]
fn schema_export_manifest_rejects_bad_hash() {
    let schema = JSONSchema::compile(&export_manifest_schema()).expect("compile schema");
    let invalid = serde_json::json!({
      "manifest_version": 1,
      "vault_id": "123e4567-e89b-12d3-a456-426614174000",
      "schema_versions": {},
      "encryption": {
        "enabled": false,
        "mode": "object_store_xchacha20poly1305",
        "key_reference": null,
        "kdf": {
          "algorithm": "argon2id",
          "memory_kib": 65536,
          "iterations": 3,
          "parallelism": 1,
          "salt_id": "vault-kdf-salt-v1"
        }
      },
      "db_encryption": {
        "enabled": false,
        "mode": "sqlcipher_v4",
        "key_reference": null,
        "kdf": {
          "algorithm": "pbkdf2_hmac_sha512"
        }
      },
      "recovery_escrow": {
        "enabled": false,
        "provider": "none",
        "providers": [],
        "updated_at_ms": null,
        "descriptor": null,
        "escrow_descriptors": []
      },
      "packaging": {
        "format": "folder",
        "zip_policy": {
          "compression": "stored",
          "mtime": "1980-01-01T00:00:00Z",
          "file_mode": "0644"
        }
      },
      "chunking_config_hash": "not-a-hash",
      "db": { "relative_path": "db/knowledge.sqlite", "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
      "objects": [],
      "indexes": {}
    });
    assert!(!schema.is_valid(&invalid));
}

#[test]
fn schema_export_manifest_rejects_non_uuid_vault_id() {
    let schema = JSONSchema::compile(&export_manifest_schema()).expect("compile schema");
    let invalid = serde_json::json!({
      "manifest_version": 1,
      "vault_id": "not-a-uuid",
      "schema_versions": {},
      "encryption": {
        "enabled": false,
        "mode": "object_store_xchacha20poly1305",
        "key_reference": null,
        "kdf": {
          "algorithm": "argon2id",
          "memory_kib": 65536,
          "iterations": 3,
          "parallelism": 1,
          "salt_id": "vault-kdf-salt-v1"
        }
      },
      "db_encryption": {
        "enabled": false,
        "mode": "sqlcipher_v4",
        "key_reference": null,
        "kdf": {
          "algorithm": "pbkdf2_hmac_sha512"
        }
      },
      "recovery_escrow": {
        "enabled": false,
        "provider": "none",
        "providers": [],
        "updated_at_ms": null,
        "descriptor": null,
        "escrow_descriptors": []
      },
      "packaging": {
        "format": "folder",
        "zip_policy": {
          "compression": "stored",
          "mtime": "1980-01-01T00:00:00Z",
          "file_mode": "0644"
        }
      },
      "chunking_config_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "db": { "relative_path": "db/knowledge.sqlite", "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
      "objects": [],
      "indexes": {}
    });
    assert!(!schema.is_valid(&invalid));
}
