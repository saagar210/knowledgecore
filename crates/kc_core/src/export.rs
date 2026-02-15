use crate::app_error::{AppError, AppResult};
use crate::canon_json::to_canonical_bytes;
use crate::chunking::{default_chunking_config_v1, hash_chunking_config};
use crate::hashing::blake3_hex_prefixed;
use crate::recovery_escrow::{
    normalize_escrow_descriptors, provider_priority, RecoveryEscrowDescriptorV2,
};
use crate::vault::{vault_open, vault_paths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_vectors: bool,
    #[serde(default)]
    pub as_zip: bool,
}

fn zip_fixed_time() -> zip::DateTime {
    zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("valid fixed zip timestamp")
}

fn create_deterministic_zip(bundle_dir: &Path, zip_path: &Path) -> AppResult<()> {
    let file = fs::File::create(zip_path).map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed creating deterministic zip file",
            false,
            serde_json::json!({ "error": e.to_string(), "path": zip_path }),
        )
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(bundle_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    files.sort();

    for file_path in files {
        let rel = rel_for_path(bundle_dir, &file_path)?;
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .last_modified_time(zip_fixed_time())
            .unix_permissions(0o644);
        zip.start_file(&rel, options).map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed starting deterministic zip entry",
                false,
                serde_json::json!({ "error": e.to_string(), "entry": rel }),
            )
        })?;

        let bytes = fs::read(&file_path).map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed reading bundle file for zip",
                false,
                serde_json::json!({ "error": e.to_string(), "path": file_path }),
            )
        })?;
        zip.write_all(&bytes).map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed writing deterministic zip entry bytes",
                false,
                serde_json::json!({ "error": e.to_string(), "entry": rel }),
            )
        })?;
    }

    zip.finish().map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed finalizing deterministic zip",
            false,
            serde_json::json!({ "error": e.to_string(), "path": zip_path }),
        )
    })?;

    Ok(())
}

fn rel_for_path(base: &Path, path: &Path) -> AppResult<String> {
    path.strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed deriving relative path for export entry",
                false,
                serde_json::json!({ "error": e.to_string(), "base": base, "path": path }),
            )
        })
}

fn recovery_escrow_export_block(conn: &rusqlite::Connection) -> AppResult<serde_json::Value> {
    let mut stmt = conn
        .prepare(
            "SELECT provider_id, enabled, descriptor_json, updated_at_ms
             FROM recovery_escrow_configs
             ORDER BY provider_id ASC",
        )
        .map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed preparing recovery escrow export query",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? != 0,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed querying recovery escrow export rows",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut descriptors = Vec::new();
    let mut providers = Vec::new();
    let mut updated_at_ms: Option<i64> = None;
    for row in rows {
        let (provider_id, enabled, descriptor_json, provider_updated_at_ms) = row.map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed decoding recovery escrow export row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        if !enabled {
            continue;
        }

        let descriptor_value = serde_json::from_str::<serde_json::Value>(&descriptor_json)
            .map_err(|e| {
                AppError::new(
                    "KC_EXPORT_FAILED",
                    "export",
                    "failed parsing recovery escrow descriptor json",
                    false,
                    serde_json::json!({ "error": e.to_string(), "provider": provider_id }),
                )
            })?;
        let Some(descriptor_obj) = descriptor_value.as_object() else {
            continue;
        };
        let has_descriptor_shape = descriptor_obj.contains_key("provider")
            && descriptor_obj.contains_key("provider_ref")
            && descriptor_obj.contains_key("key_id")
            && descriptor_obj.contains_key("wrapped_at_ms");
        if !has_descriptor_shape {
            // Provider-level config rows can exist before any escrow write has produced
            // a concrete descriptor. Those rows are intentionally omitted.
            continue;
        }
        let descriptor = serde_json::from_value::<RecoveryEscrowDescriptorV2>(descriptor_value)
            .map_err(|e| {
                AppError::new(
                    "KC_EXPORT_FAILED",
                    "export",
                    "failed decoding recovery escrow descriptor json",
                    false,
                    serde_json::json!({ "error": e.to_string(), "provider": provider_id }),
                )
            })?;
        if descriptor.provider != provider_id {
            return Err(AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "recovery escrow descriptor provider does not match provider id",
                false,
                serde_json::json!({
                    "provider_id": provider_id,
                    "descriptor_provider": descriptor.provider
                }),
            ));
        }

        providers.push(provider_id);
        descriptors.push(descriptor);
        updated_at_ms = Some(updated_at_ms.map_or(provider_updated_at_ms, |current| {
            current.max(provider_updated_at_ms)
        }));
    }

    if descriptors.is_empty() {
        return Ok(serde_json::json!({
            "enabled": false,
            "provider": "none",
            "providers": [],
            "updated_at_ms": serde_json::Value::Null,
            "descriptor": serde_json::Value::Null,
            "escrow_descriptors": []
        }));
    }

    normalize_escrow_descriptors(&mut descriptors);
    providers.sort_by(|a, b| {
        provider_priority(a)
            .cmp(&provider_priority(b))
            .then_with(|| a.cmp(b))
    });
    providers.dedup();

    let primary_provider = if providers.len() == 1 {
        providers[0].clone()
    } else {
        "multi".to_string()
    };
    let primary_descriptor = serde_json::to_value(&descriptors[0]).map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed serializing primary recovery escrow descriptor",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let descriptors_value = serde_json::to_value(&descriptors).map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed serializing recovery escrow descriptor list",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    Ok(serde_json::json!({
        "enabled": true,
        "provider": primary_provider,
        "providers": providers,
        "updated_at_ms": updated_at_ms,
        "descriptor": primary_descriptor,
        "escrow_descriptors": descriptors_value
    }))
}

pub fn export_bundle(
    vault_path: &Path,
    export_dir: &Path,
    opts: &ExportOptions,
    now_ms: i64,
) -> AppResult<PathBuf> {
    let vault = vault_open(vault_path)?;
    let paths = vault_paths(vault_path);

    let bundle_dir = export_dir.join(format!("export_{}", now_ms));
    fs::create_dir_all(&bundle_dir).map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed to create export bundle directory",
            false,
            serde_json::json!({ "error": e.to_string(), "path": bundle_dir }),
        )
    })?;

    let db_src = vault_path.join(&vault.db.relative_path);
    let db_rel = PathBuf::from(&vault.db.relative_path);
    let db_dst = bundle_dir.join(&db_rel);
    if let Some(parent) = db_dst.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed to create export db directory",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }

    fs::copy(&db_src, &db_dst).map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed to copy sqlite db",
            false,
            serde_json::json!({ "error": e.to_string(), "from": db_src, "to": db_dst }),
        )
    })?;

    let db_hash = blake3_hex_prefixed(&fs::read(&db_dst).map_err(|e| {
        AppError::new(
            "KC_EXPORT_FAILED",
            "export",
            "failed reading copied db bytes",
            false,
            serde_json::json!({ "error": e.to_string(), "path": db_dst }),
        )
    })?);

    let conn = crate::db::open_db(&db_src)?;
    let mut stmt = conn
        .prepare("SELECT object_hash, bytes FROM objects ORDER BY object_hash ASC")
        .map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed querying objects",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    let mut objects = Vec::new();
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed iterating objects",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    for row in rows {
        let (hash, bytes) = row.map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed reading object row",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        let prefix = hash.strip_prefix("blake3:").ok_or_else(|| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "invalid object hash",
                false,
                serde_json::json!({ "hash": hash }),
            )
        })?[0..2]
            .to_string();

        let rel = format!("store/objects/{}/{}", prefix, hash);
        let src = paths.objects_dir.join(prefix).join(&hash);
        let dst = bundle_dir.join(&rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_EXPORT_FAILED",
                    "export",
                    "failed creating object destination directory",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;
        }
        let stored_bytes = fs::read(&src).map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed reading source object file",
                false,
                serde_json::json!({ "error": e.to_string(), "path": src }),
            )
        })?;

        fs::write(&dst, &stored_bytes).map_err(|e| {
            AppError::new(
                "KC_EXPORT_FAILED",
                "export",
                "failed copying object file",
                false,
                serde_json::json!({ "error": e.to_string(), "from": src, "to": dst }),
            )
        })?;

        objects.push(serde_json::json!({
            "relative_path": rel,
            "hash": hash,
            "storage_hash": blake3_hex_prefixed(&stored_bytes),
            "encrypted": crate::object_store::is_encrypted_payload(&stored_bytes),
            "bytes": bytes
        }));
    }

    objects.sort_by(|a, b| {
        let ah = a.get("hash").and_then(|x| x.as_str()).unwrap_or_default();
        let bh = b.get("hash").and_then(|x| x.as_str()).unwrap_or_default();
        let ap = a
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        let bp = b
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        ah.cmp(bh).then(ap.cmp(bp))
    });

    let recovery_escrow = recovery_escrow_export_block(&conn)?;

    let mut vectors = Vec::new();
    if opts.include_vectors && paths.vectors_dir.exists() {
        let mut vector_paths: Vec<_> = walkdir::WalkDir::new(&paths.vectors_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();
        vector_paths.sort();

        for src in vector_paths {
            let rel_src = rel_for_path(vault_path, &src)?;
            let dst = bundle_dir.join(&rel_src);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::new(
                        "KC_EXPORT_FAILED",
                        "export",
                        "failed creating vectors destination directory",
                        false,
                        serde_json::json!({ "error": e.to_string(), "path": parent }),
                    )
                })?;
            }
            fs::copy(&src, &dst).map_err(|e| {
                AppError::new(
                    "KC_EXPORT_FAILED",
                    "export",
                    "failed copying vector index file",
                    false,
                    serde_json::json!({ "error": e.to_string(), "from": src, "to": dst }),
                )
            })?;
            let bytes = fs::read(&dst).map_err(|e| {
                AppError::new(
                    "KC_EXPORT_FAILED",
                    "export",
                    "failed reading copied vector index file",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": dst }),
                )
            })?;
            vectors.push(serde_json::json!({
                "relative_path": rel_src,
                "hash": blake3_hex_prefixed(&bytes),
                "bytes": bytes.len(),
            }));
        }
    }
    vectors.sort_by(|a, b| {
        let ap = a
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        let bp = b
            .get("relative_path")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        ap.cmp(bp)
    });

    let chunking_config_hash = hash_chunking_config(&default_chunking_config_v1())?;

    let manifest = serde_json::json!({
        "manifest_version": 1,
        "vault_id": vault.vault_id,
        "schema_versions": {
            "vault": 3,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "encryption": {
            "enabled": vault.encryption.enabled,
            "mode": vault.encryption.mode,
            "key_reference": vault.encryption.key_reference,
            "kdf": {
                "algorithm": vault.encryption.kdf.algorithm,
                "memory_kib": vault.encryption.kdf.memory_kib,
                "iterations": vault.encryption.kdf.iterations,
                "parallelism": vault.encryption.kdf.parallelism,
                "salt_id": vault.encryption.kdf.salt_id,
            }
        },
        "db_encryption": {
            "enabled": vault.db_encryption.enabled,
            "mode": vault.db_encryption.mode,
            "key_reference": vault.db_encryption.key_reference,
            "kdf": {
                "algorithm": vault.db_encryption.kdf.algorithm,
            }
        },
        "recovery_escrow": recovery_escrow,
        "toolchain_registry": {
            "pdfium": vault.toolchain.pdfium.identity,
            "tesseract": vault.toolchain.tesseract.identity,
        },
        "chunking_config_id": vault.defaults.chunking_config_id,
        "chunking_config_hash": chunking_config_hash.0,
        "embedding": {
            "model_id": vault.defaults.embedding_model_id
        },
        "packaging": {
            "format": if opts.as_zip { "zip" } else { "folder" },
            "zip_policy": {
                "compression": "stored",
                "mtime": "1980-01-01T00:00:00Z",
                "file_mode": "0644"
            }
        },
        "db": {
            "relative_path": vault.db.relative_path,
            "hash": db_hash
        },
        "objects": objects,
        "indexes": {
            "vectors": vectors
        }
    });

    let manifest_bytes = to_canonical_bytes(&manifest)?;
    fs::write(bundle_dir.join("manifest.json"), manifest_bytes).map_err(|e| {
        AppError::new(
            "KC_EXPORT_MANIFEST_WRITE_FAILED",
            "export",
            "failed writing manifest.json",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    if opts.as_zip {
        let zip_path = export_dir.join(format!("export_{}.zip", now_ms));
        create_deterministic_zip(&bundle_dir, &zip_path)?;
        return Ok(zip_path);
    }

    Ok(bundle_dir)
}
