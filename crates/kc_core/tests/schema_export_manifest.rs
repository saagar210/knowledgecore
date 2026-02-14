use jsonschema::JSONSchema;
use kc_core::db::open_db;
use kc_core::export::{export_bundle, ExportOptions};
use kc_core::vault::vault_init;

fn export_manifest_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/export-manifest/v1",
      "type": "object",
      "required": ["manifest_version", "vault_id", "schema_versions", "chunking_config_hash", "db", "objects"],
      "properties": {
        "manifest_version": { "const": 1 },
        "vault_id": {
          "type": "string",
          "format": "uuid",
          "pattern": "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
        },
        "schema_versions": { "type": "object" },
        "toolchain_registry": { "type": "object" },
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
            "required": ["relative_path", "hash", "bytes"],
            "properties": {
              "relative_path": { "type": "string" },
              "hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
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
      "chunking_config_hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "db": { "relative_path": "db/knowledge.sqlite", "hash": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
      "objects": [],
      "indexes": {}
    });
    assert!(!schema.is_valid(&invalid));
}
