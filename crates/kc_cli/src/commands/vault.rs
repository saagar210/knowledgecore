use kc_core::app_error::{AppError, AppResult};
use kc_core::db::open_db;
use kc_core::vault::{vault_open, vault_paths};
use std::path::Path;

pub fn run_verify(vault_path: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let paths = vault_paths(Path::new(vault_path));
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;

    let db_integrity: String = conn
        .query_row("PRAGMA integrity_check(1)", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "vault",
                "failed running sqlite integrity_check",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    if db_integrity.to_lowercase() != "ok" {
        return Err(AppError::new(
            "KC_DB_INTEGRITY_FAILED",
            "vault",
            "sqlite integrity_check failed",
            false,
            serde_json::json!({ "result": db_integrity }),
        ));
    }

    if !paths.objects_dir.exists() || !paths.vectors_dir.exists() {
        return Err(AppError::new(
            "KC_VAULT_JSON_INVALID",
            "vault",
            "vault directories are missing",
            false,
            serde_json::json!({
                "objects_dir": paths.objects_dir,
                "vectors_dir": paths.vectors_dir
            }),
        ));
    }

    let object_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM objects", [], |row| row.get(0))
        .unwrap_or(0);
    let doc_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM docs", [], |row| row.get(0))
        .unwrap_or(0);
    let event_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .unwrap_or(0);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "vault_id": vault.vault_id,
            "counts": {
                "objects": object_count,
                "docs": doc_count,
                "events": event_count
            }
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}
