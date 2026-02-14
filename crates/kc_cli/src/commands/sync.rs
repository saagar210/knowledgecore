use kc_core::app_error::AppResult;
use kc_core::db::open_db;
use kc_core::sync::{sync_pull, sync_push, sync_status};
use kc_core::vault::vault_open;
use std::path::Path;

pub fn run_status(vault_path: &str, target_path: &str) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let status = sync_status(&conn, Path::new(target_path))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_push(vault_path: &str, target_path: &str, now_ms: i64) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let out = sync_push(&conn, Path::new(vault_path), Path::new(target_path), now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_pull(vault_path: &str, target_path: &str, now_ms: i64) -> AppResult<()> {
    let vault = vault_open(Path::new(vault_path))?;
    let conn = open_db(&Path::new(vault_path).join(vault.db.relative_path))?;
    let out = sync_pull(&conn, Path::new(vault_path), Path::new(target_path), now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}
