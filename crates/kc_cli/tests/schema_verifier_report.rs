use jsonschema::JSONSchema;
use kc_cli::verifier::{CheckedCounts, VerifyErrorEntry, VerifyReportV1};
use serde_json::json;

fn verifier_report_schema() -> serde_json::Value {
    json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/verifier-report/v1",
      "type": "object",
      "required": [
        "report_version",
        "status",
        "exit_code",
        "errors",
        "checked"
      ],
      "properties": {
        "report_version": { "const": 1 },
        "status": { "type": "string", "enum": ["ok", "failed"] },
        "exit_code": { "type": "integer" },
        "errors": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["code", "path"],
            "properties": {
              "code": { "type": "string" },
              "path": { "type": "string" },
              "expected": { "type": ["string", "null"] },
              "actual": { "type": ["string", "null"] }
            },
            "additionalProperties": false
          }
        },
        "checked": {
          "type": "object",
          "required": ["objects", "indexes"],
          "properties": {
            "objects": { "type": "integer" },
            "indexes": { "type": "integer" }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_verifier_report_accepts_valid_payload() {
    let schema = JSONSchema::compile(&verifier_report_schema()).expect("compile verifier schema");
    let report = VerifyReportV1 {
        report_version: 1,
        status: "ok".to_string(),
        exit_code: 0,
        errors: vec![VerifyErrorEntry {
            code: "OBJECT_HASH_MISMATCH".to_string(),
            path: "store/objects/aa/blake3:1234".to_string(),
            expected: Some("blake3:aaaa".to_string()),
            actual: Some("blake3:bbbb".to_string()),
        }],
        checked: CheckedCounts {
            objects: 1,
            indexes: 0,
        },
    };

    let value = serde_json::to_value(report).expect("serialize report");
    assert!(schema.is_valid(&value));
}

#[test]
fn schema_verifier_report_rejects_missing_checked() {
    let schema = JSONSchema::compile(&verifier_report_schema()).expect("compile verifier schema");
    let invalid = json!({
      "report_version": 1,
      "status": "failed",
      "exit_code": 31,
      "errors": []
    });

    assert!(!schema.is_valid(&invalid));
}

#[test]
fn schema_verifier_report_accepts_recovery_escrow_metadata_error() {
    let schema = JSONSchema::compile(&verifier_report_schema()).expect("compile verifier schema");
    let report = json!({
      "report_version": 1,
      "status": "failed",
      "exit_code": 21,
      "errors": [
        {
          "code": "RECOVERY_ESCROW_METADATA_MISMATCH",
          "path": "recovery_escrow/escrow_descriptors",
          "expected": "sorted descriptors",
          "actual": "[{\"provider\":\"gcp\"}]"
        }
      ],
      "checked": { "objects": 3, "indexes": 1 }
    });
    assert!(schema.is_valid(&report));
}
