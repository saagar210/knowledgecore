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

fn internal_error(path: String, message: String, checked: CheckedCounts) -> (i64, VerifyReportV1) {
    report_for(
        60,
        vec![VerifyErrorEntry {
            code: "INTERNAL_ERROR".to_string(),
            path,
            expected: None,
            actual: Some(message),
        }],
        checked,
    )
}

fn manifest_schema() -> Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/export-manifest/v1",
      "type": "object",
      "required": ["manifest_version", "vault_id", "schema_versions", "encryption", "chunking_config_hash", "db", "objects"],
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

    let encryption_enabled = manifest
        .get("encryption")
        .and_then(|x| x.get("enabled"))
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let objects = manifest
        .get("objects")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let vectors = manifest
        .get("indexes")
        .and_then(|x| x.get("vectors"))
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    for obj in &objects {
        let path = obj
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let expected_storage_hash = obj
            .get("storage_hash")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let expected_encrypted = obj
            .get("encrypted")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);

        let abs = bundle_path.join(&path);
        if !abs.exists() {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_MISSING".to_string(),
                path: path.clone(),
                expected: Some(expected_storage_hash),
                actual: None,
            });
            continue;
        }

        let bytes = match fs::read(&abs) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Ok(internal_error(
                    path.clone(),
                    format!("failed to read object during verification: {}", e),
                    CheckedCounts {
                        objects: objects.len() as i64,
                        indexes: vectors.len() as i64,
                    },
                ))
            }
        };
        let actual_hash = blake3_hex_prefixed(&bytes);
        let actual_encrypted = kc_core::object_store::is_encrypted_payload(&bytes);

        if actual_hash != expected_storage_hash {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_HASH_MISMATCH".to_string(),
                path: path.clone(),
                expected: Some(expected_storage_hash),
                actual: Some(actual_hash),
            });
        }
        if actual_encrypted != expected_encrypted {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_ENCRYPTION_MISMATCH".to_string(),
                path: path.clone(),
                expected: Some(expected_encrypted.to_string()),
                actual: Some(actual_encrypted.to_string()),
            });
        }
        if encryption_enabled && !expected_encrypted {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_ENCRYPTION_MISMATCH".to_string(),
                path: path.clone(),
                expected: Some("true".to_string()),
                actual: Some("false".to_string()),
            });
        } else if !encryption_enabled && expected_encrypted {
            errors.push(VerifyErrorEntry {
                code: "OBJECT_ENCRYPTION_MISMATCH".to_string(),
                path: path.clone(),
                expected: Some("false".to_string()),
                actual: Some("true".to_string()),
            });
        }
    }

    for idx in &vectors {
        let path = idx
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string();
        let expected_hash = idx
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

        let bytes = match fs::read(&abs) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Ok(internal_error(
                    path,
                    format!("failed to read index file during verification: {}", e),
                    CheckedCounts {
                        objects: objects.len() as i64,
                        indexes: vectors.len() as i64,
                    },
                ))
            }
        };
        let actual_hash = blake3_hex_prefixed(&bytes);

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
    } else if errors
        .iter()
        .any(|e| e.code == "OBJECT_HASH_MISMATCH" || e.code == "OBJECT_ENCRYPTION_MISMATCH")
    {
        41
    } else {
        60
    };

    Ok(report_for(
        code,
        errors,
        CheckedCounts {
            objects: objects.len() as i64,
            indexes: vectors.len() as i64,
        },
    ))
}
