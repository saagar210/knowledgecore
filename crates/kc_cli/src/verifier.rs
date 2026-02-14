use jsonschema::JSONSchema;
use kc_core::app_error::{AppError, AppResult};
use kc_core::hashing::blake3_hex_prefixed;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn report_for(
    exit_code: i64,
    mut errors: Vec<VerifyErrorEntry>,
    checked: CheckedCounts,
) -> (i64, VerifyReportV1) {
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
      "required": ["manifest_version", "vault_id", "schema_versions", "encryption", "packaging", "chunking_config_hash", "db", "objects"],
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

#[allow(dead_code)]
fn sync_head_schema() -> Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/sync-head/v2",
      "type": "object",
      "required": ["schema_version", "snapshot_id", "manifest_hash", "created_at_ms"],
      "properties": {
        "schema_version": { "type": "integer", "enum": [1, 2] },
        "snapshot_id": { "type": "string" },
        "manifest_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "created_at_ms": { "type": "integer" },
        "trust": {
          "type": ["object", "null"],
          "required": ["model", "fingerprint", "updated_at_ms"],
          "properties": {
            "model": { "type": "string", "const": "passphrase_v1" },
            "fingerprint": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
            "updated_at_ms": { "type": "integer" }
          },
          "additionalProperties": false
        }
      },
      "allOf": [
        {
          "if": { "properties": { "schema_version": { "const": 2 } }, "required": ["schema_version"] },
          "then": { "required": ["trust"] }
        }
      ],
      "additionalProperties": false
    })
}

#[allow(dead_code)]
pub fn verify_sync_head_payload(sync_head_json: &[u8]) -> AppResult<()> {
    let payload: Value = serde_json::from_slice(sync_head_json).map_err(|e| {
        AppError::new(
            "KC_VERIFY_FAILED",
            "verify",
            "failed parsing sync head payload",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let schema = JSONSchema::compile(&sync_head_schema()).map_err(|e| {
        AppError::new(
            "KC_VERIFY_FAILED",
            "verify",
            "failed compiling sync head schema",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    if let Some(first_error) = schema.validate(&payload).err().and_then(|mut e| e.next()) {
        return Err(AppError::new(
            "KC_VERIFY_FAILED",
            "verify",
            "sync head payload failed schema validation",
            false,
            serde_json::json!({
                "instance_path": first_error.instance_path.to_string(),
                "schema_path": first_error.schema_path.to_string(),
                "error": first_error.to_string(),
            }),
        ));
    }
    Ok(())
}

fn verify_folder_bundle(bundle_path: &Path, expected_packaging_format: &str) -> AppResult<(i64, VerifyReportV1)> {
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
                CheckedCounts {
                    objects: 0,
                    indexes: 0,
                },
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
                CheckedCounts {
                    objects: 0,
                    indexes: 0,
                },
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
            CheckedCounts {
                objects: 0,
                indexes: 0,
            },
        ));
    }

    if manifest
        .get("packaging")
        .and_then(|x| x.get("format"))
        .and_then(|x| x.as_str())
        != Some(expected_packaging_format)
    {
        return Ok(report_for(
            21,
            vec![VerifyErrorEntry {
                code: "MANIFEST_SCHEMA_INVALID".to_string(),
                path: "/packaging/format".to_string(),
                expected: Some(expected_packaging_format.to_string()),
                actual: manifest
                    .get("packaging")
                    .and_then(|x| x.get("format"))
                    .and_then(|x| x.as_str())
                    .map(|x| x.to_string()),
            }],
            CheckedCounts {
                objects: 0,
                indexes: 0,
            },
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
    let db_rel = db
        .get("relative_path")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
    let db_hash_expected = db
        .get("hash")
        .and_then(|x| x.as_str())
        .unwrap_or_default();
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

fn is_zip_time_normalized(dt: zip::DateTime) -> bool {
    dt.year() == 1980
        && dt.month() == 1
        && dt.day() == 1
        && dt.hour() == 0
        && dt.minute() == 0
        && dt.second() == 0
}

fn safe_zip_relative_path(name: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in Path::new(name).components() {
        match component {
            Component::Normal(part) => out.push(part),
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

fn temporary_extract_root() -> AppResult<PathBuf> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("kc_verify_zip_{}_{}", std::process::id(), nonce));
    fs::create_dir_all(&dir).map_err(|e| {
        AppError::new(
            "KC_VERIFY_FAILED",
            "verify",
            "failed creating temporary zip verification directory",
            false,
            serde_json::json!({ "error": e.to_string(), "path": dir }),
        )
    })?;
    Ok(dir)
}

fn verify_zip_bundle(bundle_zip: &Path) -> AppResult<(i64, VerifyReportV1)> {
    let file = match fs::File::open(bundle_zip) {
        Ok(file) => file,
        Err(e) => {
            return Ok(report_for(
                20,
                vec![VerifyErrorEntry {
                    code: "MANIFEST_INVALID_JSON".to_string(),
                    path: bundle_zip.display().to_string(),
                    expected: None,
                    actual: Some(e.to_string()),
                }],
                CheckedCounts {
                    objects: 0,
                    indexes: 0,
                },
            ))
        }
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            return Ok(report_for(
                21,
                vec![VerifyErrorEntry {
                    code: "MANIFEST_SCHEMA_INVALID".to_string(),
                    path: bundle_zip.display().to_string(),
                    expected: Some("valid deterministic zip archive".to_string()),
                    actual: Some(e.to_string()),
                }],
                CheckedCounts {
                    objects: 0,
                    indexes: 0,
                },
            ))
        }
    };

    let extract_root = temporary_extract_root()?;
    let mut entry_names = Vec::<String>::new();
    let mut zip_errors = Vec::<VerifyErrorEntry>::new();

    for idx in 0..archive.len() {
        let mut entry = match archive.by_index(idx) {
            Ok(entry) => entry,
            Err(e) => {
                return Ok(internal_error(
                    bundle_zip.display().to_string(),
                    format!("failed reading zip entry: {}", e),
                    CheckedCounts {
                        objects: 0,
                        indexes: 0,
                    },
                ))
            }
        };

        let name = entry.name().to_string();
        if name.ends_with('/') {
            continue;
        }
        entry_names.push(name.clone());

        if entry.compression() != zip::CompressionMethod::Stored {
            zip_errors.push(VerifyErrorEntry {
                code: "ZIP_METADATA_INVALID".to_string(),
                path: name.clone(),
                expected: Some("compression=stored".to_string()),
                actual: Some(format!("compression={:?}", entry.compression())),
            });
        }

        if !is_zip_time_normalized(entry.last_modified()) {
            zip_errors.push(VerifyErrorEntry {
                code: "ZIP_METADATA_INVALID".to_string(),
                path: name.clone(),
                expected: Some("mtime=1980-01-01T00:00:00Z".to_string()),
                actual: Some(format!(
                    "mtime={:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                    entry.last_modified().year(),
                    entry.last_modified().month(),
                    entry.last_modified().day(),
                    entry.last_modified().hour(),
                    entry.last_modified().minute(),
                    entry.last_modified().second()
                )),
            });
        }

        let mode = entry.unix_mode().unwrap_or(0o644) & 0o777;
        if mode != 0o644 {
            zip_errors.push(VerifyErrorEntry {
                code: "ZIP_METADATA_INVALID".to_string(),
                path: name.clone(),
                expected: Some("mode=0644".to_string()),
                actual: Some(format!("mode={:04o}", mode)),
            });
        }

        let rel = match safe_zip_relative_path(&name) {
            Some(path) => path,
            None => {
                zip_errors.push(VerifyErrorEntry {
                    code: "ZIP_METADATA_INVALID".to_string(),
                    path: name,
                    expected: Some("relative path without traversal".to_string()),
                    actual: Some("invalid path".to_string()),
                });
                continue;
            }
        };

        let out_path = extract_root.join(rel);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_VERIFY_FAILED",
                    "verify",
                    "failed creating extraction directory",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }

        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes).map_err(|e| {
            AppError::new(
                "KC_VERIFY_FAILED",
                "verify",
                "failed reading zip entry bytes",
                false,
                serde_json::json!({ "error": e.to_string(), "entry": name }),
            )
        })?;
        fs::write(&out_path, &bytes).map_err(|e| {
            AppError::new(
                "KC_VERIFY_FAILED",
                "verify",
                "failed writing extracted zip entry",
                false,
                serde_json::json!({ "error": e.to_string(), "path": out_path }),
            )
        })?;
    }

    let mut sorted = entry_names.clone();
    sorted.sort();
    if entry_names != sorted {
        zip_errors.push(VerifyErrorEntry {
            code: "ZIP_METADATA_INVALID".to_string(),
            path: bundle_zip.display().to_string(),
            expected: Some("entries sorted lexicographically".to_string()),
            actual: Some("archive entry order is not deterministic".to_string()),
        });
    }

    if !zip_errors.is_empty() {
        let _ = fs::remove_dir_all(&extract_root);
        return Ok(report_for(
            21,
            zip_errors,
            CheckedCounts {
                objects: 0,
                indexes: 0,
            },
        ));
    }

    let out = verify_folder_bundle(&extract_root, "zip");
    let _ = fs::remove_dir_all(&extract_root);
    out
}

pub fn verify_bundle(bundle_path: &Path) -> AppResult<(i64, VerifyReportV1)> {
    if bundle_path.is_file() {
        let is_zip = bundle_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);
        if is_zip {
            return verify_zip_bundle(bundle_path);
        }
    }

    verify_folder_bundle(bundle_path, "folder")
}
