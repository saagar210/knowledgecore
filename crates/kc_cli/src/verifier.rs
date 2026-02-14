use kc_core::app_error::{AppError, AppResult};
use kc_core::hashing::blake3_hex_prefixed;
use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReportV1 {
    pub report_version: i64,
    pub status: String,
    pub exit_code: i64,
    pub errors: Vec<VerifyErrorEntry>,
    pub checked: CheckedCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyErrorEntry {
    pub code: String,
    pub path: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedCounts {
    pub objects: i64,
    pub indexes: i64,
}

fn report_for(exit_code: i64, mut errors: Vec<VerifyErrorEntry>, checked: CheckedCounts) -> (i64, VerifyReportV1) {
    errors.sort_by(|a, b| a.code.cmp(&b.code).then(a.path.cmp(&b.path)));
    let status = if exit_code == 0 { "ok" } else { "failed" };
    (
        exit_code,
        VerifyReportV1 {
            report_version: 1,
            status: status.to_string(),
            exit_code,
            errors,
            checked,
        },
    )
}

fn manifest_schema() -> Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/export-manifest/v1",
      "type": "object",
      "required": ["manifest_version", "vault_id", "schema_versions", "chunking_config_hash", "db", "objects"],
      "properties": {
        "manifest_version": { "const": 1 },
        "vault_id": { "type": "string" },
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

pub fn verify_bundle(bundle_path: &Path) -> AppResult<(i64, VerifyReportV1)> {
    let manifest_path = bundle_path.join("manifest.json");
    let raw = match fs::read_to_string(&manifest_path) {
        Ok(v) => v,
        Err(e) => {
            return Ok(report_for(
                20,
                vec![VerifyErrorEntry {
                    code: "MANIFEST_INVALID_JSON".to_string(),
                    path: manifest_path.display().to_string(),
                    expected: None,
                    actual: Some(e.to_string()),
                }],
                CheckedCounts { objects: 0, indexes: 0 },
            ))
        }
    };

    let manifest: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return Ok(report_for(
                20,
                vec![VerifyErrorEntry {
                    code: "MANIFEST_INVALID_JSON".to_string(),
                    path: manifest_path.display().to_string(),
                    expected: None,
                    actual: Some(e.to_string()),
                }],
                CheckedCounts { objects: 0, indexes: 0 },
            ))
        }
    };

    let schema = JSONSchema::compile(&manifest_schema()).map_err(|e| {
        AppError::new(
            "KC_VERIFY_FAILED",
            "verify",
            "failed compiling manifest schema",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let mut validation_errors = schema.validate(&manifest).err().into_iter().flatten();
    if let Some(first_error) = validation_errors.next() {
        return Ok(report_for(
            21,
            vec![VerifyErrorEntry {
                code: "MANIFEST_SCHEMA_INVALID".to_string(),
                path: first_error.instance_path.to_string(),
                expected: Some(first_error.schema_path.to_string()),
                actual: Some(first_error.to_string()),
            }],
            CheckedCounts { objects: 0, indexes: 0 },
        ));
    }

    let mut errors = Vec::new();

    let db = manifest.get("db").and_then(|x| x.as_object()).ok_or_else(|| {
        AppError::new(
            "KC_VERIFY_FAILED",
            "verify",
            "manifest db field must be object",
            false,
            serde_json::json!({}),
        )
    })?;
    let db_rel = db.get("relative_path").and_then(|x| x.as_str()).unwrap_or_default();
    let db_hash_expected = db.get("hash").and_then(|x| x.as_str()).unwrap_or_default();
    let db_path = bundle_path.join(db_rel);
    match fs::read(&db_path) {
        Ok(bytes) => {
            let actual = blake3_hex_prefixed(&bytes);
            if actual != db_hash_expected {
                errors.push(VerifyErrorEntry {
                    code: "DB_HASH_MISMATCH".to_string(),
                    path: db_rel.to_string(),
                    expected: Some(db_hash_expected.to_string()),
                    actual: Some(actual),
                });
            }
        }
        Err(_) => errors.push(VerifyErrorEntry {
            code: "DB_HASH_MISMATCH".to_string(),
            path: db_rel.to_string(),
            expected: Some(db_hash_expected.to_string()),
            actual: None,
        }),
    }

    let objects = manifest
        .get("objects")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    for obj in &objects {
        let path = obj
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let expected_hash = obj
            .get("hash")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();

        let abs = bundle_path.join(&path);
        if !abs.exists() {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_MISSING".to_string(),
                path,
                expected: Some(expected_hash),
                actual: None,
            });
            continue;
        }

        let actual_hash = blake3_hex_prefixed(&fs::read(&abs).map_err(|e| {
            AppError::new(
                "KC_VERIFY_FAILED",
                "verify",
                "failed to read object during verification",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?);

        if actual_hash != expected_hash {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_HASH_MISMATCH".to_string(),
                path,
                expected: Some(expected_hash),
                actual: Some(actual_hash),
            });
        }
    }

    let code = if errors.is_empty() {
        0
    } else if errors.iter().any(|e| e.code == "DB_HASH_MISMATCH") {
        31
    } else if errors.iter().any(|e| e.code == "OBJECT_MISSING") {
        40
    } else if errors.iter().any(|e| e.code == "OBJECT_HASH_MISMATCH") {
        41
    } else {
        60
    };

    Ok(report_for(
        code,
        errors,
        CheckedCounts {
            objects: objects.len() as i64,
            indexes: 0,
        },
    ))
}
