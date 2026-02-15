use kc_core::app_error::{AppError, AppResult};
use kc_core::db::open_db;
use kc_core::vault::{vault_open, vault_paths};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn run_gc(vault_path: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let paths = vault_paths(Path::new(vault_path));
    let db = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;

    let mut stmt = db.prepare("SELECT object_hash FROM objects").map_err(|e| {
        AppError::new(
            "KC_DB_INTEGRITY_FAILED",
            "gc",
            "failed preparing object query",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let existing: HashSet<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "gc",
                "failed querying object hashes",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?
        .map(|row| row.unwrap_or_default())
        .collect();

    let mut removed = 0usize;
    if paths.objects_dir.exists() {
        for entry in walkdir::WalkDir::new(&paths.objects_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let name = entry.file_name().to_string_lossy().to_string();
            if !existing.contains(&name) {
                fs::remove_file(entry.path()).map_err(|e| {
                    AppError::new(
                        "KC_DB_INTEGRITY_FAILED",
                        "gc",
                        "failed removing orphan object file",
                        false,
                        serde_json::json!({ "error": e.to_string(), "path": entry.path() }),
                    )
                })?;
                removed += 1;
            }
        }
    }

    println!("gc removed {} orphan objects", removed);
    Ok(())
}
