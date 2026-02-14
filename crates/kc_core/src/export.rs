use crate::app_error::{AppError, AppResult};
use crate::chunking::{default_chunking_config_v1, hash_chunking_config};
use crate::canon_json::to_canonical_bytes;
use crate::hashing::blake3_hex_prefixed;
use crate::vault::{vault_open, vault_paths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_vectors: bool,
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

pub fn export_bundle(vault_path: &Path, export_dir: &Path, opts: &ExportOptions, now_ms: i64) -> AppResult<PathBuf> {
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
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
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

        let prefix = hash
            .strip_prefix("blake3:")
            .ok_or_else(|| AppError::new("KC_EXPORT_FAILED", "export", "invalid object hash", false, serde_json::json!({ "hash": hash })))?
            [0..2]
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
        fs::copy(&src, &dst).map_err(|e| {
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
            "vault": 1,
            "locator": 1,
            "app_error": 1,
            "rpc": 1
        },
        "toolchain_registry": {
            "pdfium": vault.toolchain.pdfium.identity,
            "tesseract": vault.toolchain.tesseract.identity,
        },
        "chunking_config_id": vault.defaults.chunking_config_id,
        "chunking_config_hash": chunking_config_hash.0,
        "embedding": {
            "model_id": vault.defaults.embedding_model_id
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

    Ok(bundle_dir)
}
