use jsonschema::JSONSchema;

fn draft_zip_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/draft/zip-packaging-metadata/v1",
      "type": "object",
      "required": [
        "schema_version",
        "status",
        "activation_phase",
        "format",
        "entry_order",
        "timestamp_policy",
        "permission_policy"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "status": { "const": "draft" },
        "activation_phase": { "const": "N1" },
        "format": { "const": "zip" },
        "entry_order": { "type": "string" },
        "timestamp_policy": { "type": "string" },
        "permission_policy": { "type": "string" }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_draft_zip_packaging_accepts_representative_payload() {
    let schema = JSONSchema::compile(&draft_zip_schema()).expect("compile draft zip schema");
    let value = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "N1",
      "format": "zip",
      "entry_order": "lexicographic_path",
      "timestamp_policy": "fixed_epoch_ms",
      "permission_policy": "normalized_posix_mode"
    });

    assert!(schema.is_valid(&value));
}

#[test]
fn schema_draft_zip_packaging_rejects_wrong_activation_phase() {
    let schema = JSONSchema::compile(&draft_zip_schema()).expect("compile draft zip schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "N2",
      "format": "zip",
      "entry_order": "lexicographic_path",
      "timestamp_policy": "fixed_epoch_ms",
      "permission_policy": "normalized_posix_mode"
    });

    assert!(!schema.is_valid(&invalid));
}
