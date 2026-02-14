use crate::app_error::{AppError, AppResult};
use rusqlite::Connection;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const LATEST_SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone, Deserialize, Default)]
struct VaultDbEncryptionMeta {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    mode: String,
}

#[derive(Debug, Clone, Deserialize)]
struct VaultMetaForDb {
    schema_version: u32,
    #[serde(default)]
    db_encryption: VaultDbEncryptionMeta,
}

fn maybe_vault_json_path(db_path: &Path) -> Option<PathBuf> {
    let db_parent = db_path.parent()?;
    let vault_root = db_parent.parent()?;
    Some(vault_root.join("vault.json"))
}

fn read_vault_meta_for_db(db_path: &Path) -> AppResult<Option<VaultMetaForDb>> {
    let Some(vault_json_path) = maybe_vault_json_path(db_path) else {
        return Ok(None);
    };
    if !vault_json_path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(&vault_json_path).map_err(|e| {
        AppError::new(
            "KC_DB_OPEN_FAILED",
            "db",
            "failed reading vault.json while opening database",
            false,
            serde_json::json!({ "error": e.to_string(), "path": vault_json_path }),
        )
    })?;

    let meta = serde_json::from_slice::<VaultMetaForDb>(&bytes).map_err(|e| {
        AppError::new(
            "KC_DB_OPEN_FAILED",
            "db",
            "failed parsing vault.json while opening database",
            false,
            serde_json::json!({ "error": e.to_string(), "path": vault_json_path }),
        )
    })?;

    Ok(Some(meta))
}

fn apply_db_encryption_key_if_needed(conn: &Connection, db_path: &Path) -> AppResult<()> {
    let Some(meta) = read_vault_meta_for_db(db_path)? else {
        return Ok(());
    };

    // v1/v2 vaults have no DB-at-rest metadata.
    if meta.schema_version < 3 {
        return Ok(());
    }

    if !meta.db_encryption.enabled {
        return Ok(());
    }

    if meta.db_encryption.mode != "sqlcipher_v4" {
        return Err(AppError::new(
            "KC_DB_ENCRYPTION_UNSUPPORTED",
            "db",
            "unsupported db encryption mode",
            false,
            serde_json::json!({
                "mode": meta.db_encryption.mode,
                "supported": ["sqlcipher_v4"]
            }),
        ));
    }

    let passphrase = std::env::var("KC_VAULT_DB_PASSPHRASE")
        .ok()
        .or_else(|| std::env::var("KC_VAULT_PASSPHRASE").ok())
        .ok_or_else(|| {
            AppError::new(
                "KC_DB_LOCKED",
                "db",
                "database is encrypted; passphrase environment variable is required",
                false,
                serde_json::json!({
                    "accepted_env": ["KC_VAULT_DB_PASSPHRASE", "KC_VAULT_PASSPHRASE"]
                }),
            )
        })?;

    let escaped = passphrase.replace('\'', "''");
    conn.execute_batch(&format!(
        "PRAGMA key = '{}'; PRAGMA cipher_compatibility = 4;",
        escaped
    ))
    .map_err(|e| {
        AppError::new(
            "KC_DB_KEY_INVALID",
            "db",
            "failed applying sqlcipher key pragmas",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get::<_, i64>(0))
        .map_err(|e| {
            AppError::new(
                "KC_DB_KEY_INVALID",
                "db",
                "provided db passphrase is invalid",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    Ok(())
}

pub fn open_db(db_path: &Path) -> AppResult<Connection> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::new(
                "KC_DB_OPEN_FAILED",
                "db",
                "failed to create database parent directory",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }

    let conn = Connection::open(db_path).map_err(|e| {
        AppError::new(
            "KC_DB_OPEN_FAILED",
            "db",
            "failed to open sqlite database",
            false,
            serde_json::json!({ "error": e.to_string(), "path": db_path }),
        )
    })?;

    apply_db_encryption_key_if_needed(&conn, db_path)?;

    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| {
            AppError::new(
                "KC_DB_OPEN_FAILED",
                "db",
                "failed to enable foreign_keys pragma",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    apply_migrations(&conn)?;
    Ok(conn)
}

pub fn apply_migrations(conn: &Connection) -> AppResult<()> {
    let current = schema_version(conn)?;
    if current > LATEST_SCHEMA_VERSION {
        return Err(AppError::new(
            "KC_DB_SCHEMA_INCOMPATIBLE",
            "db",
            "database schema version is newer than supported",
            false,
            serde_json::json!({ "current": current, "latest": LATEST_SCHEMA_VERSION }),
        ));
    }

    if current < 1 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0001_init.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0001",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 1i64)
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to set schema user_version",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.commit().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to commit migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }

    let current_after_v1 = schema_version(conn)?;
    if current_after_v1 < 2 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0002_sync.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0002",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", LATEST_SCHEMA_VERSION)
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to set schema user_version",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.commit().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to commit migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
    }

    Ok(())
}

pub fn schema_version(conn: &Connection) -> AppResult<i64> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| {
            AppError::new(
                "KC_DB_SCHEMA_INCOMPATIBLE",
                "db",
                "failed to read schema version",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })
}
