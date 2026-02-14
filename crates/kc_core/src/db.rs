use crate::app_error::{AppError, AppResult};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

const LATEST_SCHEMA_VERSION: i64 = 2;

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
