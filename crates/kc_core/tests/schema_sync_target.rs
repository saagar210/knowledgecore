use jsonschema::JSONSchema;
use kc_core::sync_transport::SyncTargetUri;

fn sync_target_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/sync-target-uri/v2",
      "oneOf": [
        {
          "type": "object",
          "required": ["kind", "path"],
          "properties": {
            "kind": { "const": "file_path" },
            "path": { "type": "string", "minLength": 1 }
          },
          "additionalProperties": false
        },
        {
          "type": "object",
          "required": ["kind", "bucket", "prefix"],
          "properties": {
            "kind": { "const": "s3" },
            "bucket": { "type": "string", "minLength": 1 },
            "prefix": { "type": "string" }
          },
          "additionalProperties": false
        }
      ]
    })
}

#[test]
fn schema_sync_target_accepts_file_and_s3() {
    let schema = JSONSchema::compile(&sync_target_schema()).expect("compile schema");
    let file = serde_json::to_value(SyncTargetUri::parse("/tmp/sync").expect("parse file"))
        .expect("serialize file");
    let s3 = serde_json::to_value(SyncTargetUri::parse("s3://demo-bucket/kc").expect("parse s3"))
        .expect("serialize s3");
    assert!(schema.is_valid(&file));
    assert!(schema.is_valid(&s3));
}

#[test]
fn schema_sync_target_rejects_unknown_scheme() {
    let err = SyncTargetUri::parse("ftp://example/path").expect_err("scheme must fail");
    assert_eq!(err.code, "KC_SYNC_TARGET_UNSUPPORTED");
}
