use jsonschema::validator_for;
use kc_core::app_error::AppError;
use serde_json::json;

fn app_error_schema() -> serde_json::Value {
    json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/app-error/v1",
      "type": "object",
      "required": [
        "schema_version",
        "code",
        "category",
        "message",
        "retryable",
        "details"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "code": { "type": "string" },
        "category": { "type": "string" },
        "message": { "type": "string" },
        "retryable": { "type": "boolean" },
        "details": {}
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_app_error_accepts_valid_payload() {
    let schema = validator_for(&app_error_schema()).expect("compile app_error schema");
    let value = serde_json::to_value(AppError::new(
        "KC_VAULT_INIT_FAILED",
        "vault",
        "failed to initialize vault",
        false,
        json!({ "path": "/tmp/demo" }),
    ))
    .expect("serialize app_error");

    assert!(schema.is_valid(&value));
}

#[test]
fn schema_app_error_rejects_missing_code() {
    let schema = validator_for(&app_error_schema()).expect("compile app_error schema");
    let invalid = json!({
      "schema_version": 1,
      "category": "vault",
      "message": "missing code",
      "retryable": false,
      "details": {}
    });

    assert!(!schema.is_valid(&invalid));
}
