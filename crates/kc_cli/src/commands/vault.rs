use kc_core::app_error::{AppError, AppResult};
use kc_core::db::open_db;
use kc_core::rpc_service::{
    vault_encryption_enable_service, vault_encryption_migrate_service,
    vault_encryption_status_service,
};
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

fn passphrase_from_env(passphrase_env: &str) -> AppResult<String> {
    std::env::var(passphrase_env)
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            AppError::new(
                "KC_ENCRYPTION_REQUIRED",
                "encryption",
                "passphrase env var is missing or empty",
                false,
                serde_json::json!({ "passphrase_env": passphrase_env }),
            )
        })
}

pub fn run_encrypt_status(vault_path: &str) -> AppResult<()> {
    let status = vault_encryption_status_service(Path::new(vault_path))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "encryption": {
                "enabled": status.enabled,
                "mode": status.mode,
                "key_reference": status.key_reference,
                "kdf_algorithm": status.kdf_algorithm,
                "objects_total": status.objects_total,
                "objects_encrypted": status.objects_encrypted,
            }
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_encrypt_enable(vault_path: &str, passphrase_env: &str) -> AppResult<()> {
    let passphrase = passphrase_from_env(passphrase_env)?;
    let status = vault_encryption_enable_service(Path::new(vault_path), &passphrase)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "enabled": status.enabled,
            "mode": status.mode,
            "objects_total": status.objects_total,
            "objects_encrypted": status.objects_encrypted,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_encrypt_migrate(vault_path: &str, passphrase_env: &str, now_ms: i64) -> AppResult<()> {
    let passphrase = passphrase_from_env(passphrase_env)?;
    let out = vault_encryption_migrate_service(Path::new(vault_path), &passphrase, now_ms)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "event_id": out.event_id,
            "migrated_objects": out.migrated_objects,
            "already_encrypted_objects": out.already_encrypted_objects,
            "objects_total": out.status.objects_total,
            "objects_encrypted": out.status.objects_encrypted,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run_encrypt_enable, run_encrypt_migrate};
    use kc_core::db::open_db;
    use kc_core::object_store::{is_encrypted_payload, ObjectStore};
    use kc_core::rpc_service::vault_encryption_status_service;
    use kc_core::vault::vault_init;

    #[test]
    fn encrypt_enable_and_migrate_round_trip() {
        let root = tempfile::tempdir().expect("tempdir").keep();
        vault_init(&root, "demo", 1).expect("vault init");

        let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
        let store = ObjectStore::new(root.join("store/objects"));
        let hash = store.put_bytes(&conn, b"hello", 1).expect("put object");

        let env_name = format!("KC_TEST_PASSPHRASE_{}", std::process::id());
        std::env::set_var(&env_name, "test-passphrase");

        run_encrypt_enable(root.to_string_lossy().as_ref(), &env_name).expect("enable encryption");

        let status_before = vault_encryption_status_service(&root).expect("status before migrate");
        assert!(status_before.enabled);
        assert_eq!(status_before.objects_total, 1);
        assert_eq!(status_before.objects_encrypted, 0);

        run_encrypt_migrate(root.to_string_lossy().as_ref(), &env_name, 2).expect("migrate encryption");

        let status_after = vault_encryption_status_service(&root).expect("status after migrate");
        assert_eq!(status_after.objects_total, 1);
        assert_eq!(status_after.objects_encrypted, 1);

        let raw = ObjectStore::new(root.join("store/objects"))
            .raw_bytes(&hash)
            .expect("raw bytes");
        assert!(is_encrypted_payload(&raw));

        std::env::remove_var(env_name);
    }
}
