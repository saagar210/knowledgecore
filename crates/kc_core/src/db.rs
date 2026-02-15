use crate::app_error::{AppError, AppResult};
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const LATEST_SCHEMA_VERSION: i64 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbMigrationOutcome {
    Migrated,
    AlreadyEncrypted,
}

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

static DB_UNLOCK_SESSIONS: OnceLock<Mutex<HashMap<PathBuf, String>>> = OnceLock::new();

fn db_unlock_sessions() -> &'static Mutex<HashMap<PathBuf, String>> {
    DB_UNLOCK_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn normalize_session_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn maybe_vault_json_path(db_path: &Path) -> Option<PathBuf> {
    let db_parent = db_path.parent()?;
    let vault_root = db_parent.parent()?;
    Some(vault_root.join("vault.json"))
}

fn maybe_vault_root(db_path: &Path) -> Option<PathBuf> {
    maybe_vault_json_path(db_path).and_then(|p| p.parent().map(|x| x.to_path_buf()))
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

fn sql_string_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn attach_path_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn passphrase_from_env() -> Option<String> {
    std::env::var("KC_VAULT_DB_PASSPHRASE")
        .ok()
        .or_else(|| std::env::var("KC_VAULT_PASSPHRASE").ok())
}

fn passphrase_from_session(db_path: &Path) -> Option<String> {
    let vault_root = maybe_vault_root(db_path)?;
    let key = normalize_session_key(&vault_root);
    let sessions = db_unlock_sessions().lock().ok()?;
    sessions.get(&key).cloned()
}

fn validate_key_on_connection(conn: &Connection, passphrase: &str) -> AppResult<()> {
    let escaped = sql_string_literal(passphrase);
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

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        row.get::<_, i64>(0)
    })
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

fn verify_db_passphrase(db_path: &Path, passphrase: &str) -> AppResult<()> {
    let conn = Connection::open(db_path).map_err(|e| {
        AppError::new(
            "KC_DB_OPEN_FAILED",
            "db",
            "failed opening database for passphrase validation",
            false,
            serde_json::json!({ "error": e.to_string(), "path": db_path }),
        )
    })?;
    validate_key_on_connection(&conn, passphrase)
}

pub fn db_unlock(vault_path: &Path, db_path: &Path, passphrase: &str) -> AppResult<()> {
    let key = normalize_session_key(vault_path);
    verify_db_passphrase(db_path, passphrase)?;
    let mut sessions = db_unlock_sessions().lock().map_err(|_| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "db",
            "failed acquiring db unlock session lock",
            true,
            serde_json::json!({}),
        )
    })?;
    sessions.insert(key, passphrase.to_string());
    Ok(())
}

pub fn db_lock(vault_path: &Path) -> AppResult<()> {
    let key = normalize_session_key(vault_path);
    let mut sessions = db_unlock_sessions().lock().map_err(|_| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "db",
            "failed acquiring db unlock session lock",
            true,
            serde_json::json!({}),
        )
    })?;
    sessions.remove(&key);
    Ok(())
}

pub fn db_is_unlocked(vault_path: &Path) -> bool {
    let key = normalize_session_key(vault_path);
    let Ok(sessions) = db_unlock_sessions().lock() else {
        return false;
    };
    sessions.contains_key(&key)
}

pub fn migrate_db_to_sqlcipher(db_path: &Path, passphrase: &str) -> AppResult<DbMigrationOutcome> {
    let source_conn = Connection::open(db_path).map_err(|e| {
        AppError::new(
            "KC_DB_ENCRYPTION_MIGRATION_FAILED",
            "db",
            "failed opening source database for migration",
            false,
            serde_json::json!({ "error": e.to_string(), "path": db_path }),
        )
    })?;

    // If the source no longer opens as plaintext, treat as already encrypted only when the key validates.
    if source_conn
        .query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .is_err()
    {
        verify_db_passphrase(db_path, passphrase)?;
        return Ok(DbMigrationOutcome::AlreadyEncrypted);
    }

    let db_dir = db_path.parent().ok_or_else(|| {
        AppError::new(
            "KC_DB_ENCRYPTION_MIGRATION_FAILED",
            "db",
            "database path has no parent directory",
            false,
            serde_json::json!({ "path": db_path }),
        )
    })?;
    fs::create_dir_all(db_dir).map_err(|e| {
        AppError::new(
            "KC_DB_ENCRYPTION_MIGRATION_FAILED",
            "db",
            "failed creating migration directory",
            false,
            serde_json::json!({ "error": e.to_string(), "path": db_dir }),
        )
    })?;

    let tmp_path = db_path.with_extension("sqlcipher.tmp");
    let bak_path = db_path.with_extension("pre-sqlcipher.bak");
    let _ = fs::remove_file(&tmp_path);
    let _ = fs::remove_file(&bak_path);

    let pass = sql_string_literal(passphrase);
    let tmp_lit = attach_path_literal(&tmp_path);
    source_conn
        .execute_batch(&format!(
            "ATTACH DATABASE '{}' AS encrypted KEY '{}';\
             PRAGMA encrypted.cipher_compatibility = 4;\
             SELECT sqlcipher_export('encrypted');\
             DETACH DATABASE encrypted;",
            tmp_lit, pass
        ))
        .map_err(|e| {
            AppError::new(
                "KC_DB_ENCRYPTION_MIGRATION_FAILED",
                "db",
                "failed running sqlcipher export migration",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    verify_db_passphrase(&tmp_path, passphrase)?;

    fs::rename(db_path, &bak_path).map_err(|e| {
        AppError::new(
            "KC_DB_ENCRYPTION_MIGRATION_FAILED",
            "db",
            "failed rotating source database before finalizing migration",
            false,
            serde_json::json!({ "error": e.to_string(), "from": db_path, "to": bak_path }),
        )
    })?;

    let finalize = (|| -> AppResult<()> {
        fs::rename(&tmp_path, db_path).map_err(|e| {
            AppError::new(
                "KC_DB_ENCRYPTION_MIGRATION_FAILED",
                "db",
                "failed promoting encrypted database",
                false,
                serde_json::json!({ "error": e.to_string(), "from": tmp_path, "to": db_path }),
            )
        })?;
        verify_db_passphrase(db_path, passphrase)?;
        Ok(())
    })();

    match finalize {
        Ok(()) => {
            let _ = fs::remove_file(&bak_path);
            Ok(DbMigrationOutcome::Migrated)
        }
        Err(err) => {
            let _ = fs::remove_file(db_path);
            let _ = fs::rename(&bak_path, db_path);
            let _ = fs::remove_file(&tmp_path);
            Err(err)
        }
    }
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

    let passphrase = passphrase_from_session(db_path)
        .or_else(passphrase_from_env)
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

    validate_key_on_connection(conn, &passphrase)
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

        tx.pragma_update(None, "user_version", 1i64).map_err(|e| {
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

        tx.pragma_update(None, "user_version", 2i64).map_err(|e| {
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

    let current_after_v2 = schema_version(conn)?;
    if current_after_v2 < 3 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0003_lineage_overlays.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0003",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 3i64).map_err(|e| {
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

    let current_after_v3 = schema_version(conn)?;
    if current_after_v3 < 4 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0004_device_trust.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0004",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 4i64).map_err(|e| {
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

    let current_after_v4 = schema_version(conn)?;
    if current_after_v4 < 5 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0005_lineage_edit_locks.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0005",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 5i64).map_err(|e| {
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

    let current_after_v5 = schema_version(conn)?;
    if current_after_v5 < 6 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0006_trust_identity_v2.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0006",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 6i64).map_err(|e| {
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

    let current_after_v6 = schema_version(conn)?;
    if current_after_v6 < 7 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0007_recovery_escrow_v2.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0007",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 7i64)
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

    let current_after_v7 = schema_version(conn)?;
    if current_after_v7 < 8 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0008_lineage_rbac_v2.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0008",
                    false,
                    serde_json::json!({ "error": e.to_string() }),
                )
            })?;

        tx.pragma_update(None, "user_version", 8i64)
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

    let current_after_v8 = schema_version(conn)?;
    if current_after_v8 < 9 {
        let tx = conn.unchecked_transaction().map_err(|e| {
            AppError::new(
                "KC_DB_MIGRATION_FAILED",
                "db",
                "failed to begin migration transaction",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        tx.execute_batch(include_str!("../migrations/0009_trust_provider_governance.sql"))
            .map_err(|e| {
                AppError::new(
                    "KC_DB_MIGRATION_FAILED",
                    "db",
                    "failed to apply migration 0009",
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
