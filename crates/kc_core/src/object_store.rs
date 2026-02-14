use crate::app_error::{AppError, AppResult};
use crate::hashing::blake3_hex_prefixed;
use crate::types::ObjectHash;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

pub struct ObjectStore {
    pub objects_dir: PathBuf,
}

impl ObjectStore {
    pub fn new(objects_dir: PathBuf) -> Self {
        Self { objects_dir }
    }

    fn file_path_for_hash(&self, object_hash: &ObjectHash) -> AppResult<PathBuf> {
        if !object_hash.0.starts_with("blake3:") || object_hash.0.len() < 9 {
            return Err(AppError::new(
                "KC_HASH_INVALID_FORMAT",
                "object_store",
                "invalid object hash format",
                false,
                serde_json::json!({ "object_hash": object_hash.0 }),
            ));
        }
        let digest = &object_hash.0[7..];
        let prefix = &digest[0..2];
        Ok(self.objects_dir.join(prefix).join(&object_hash.0))
    }

    pub fn put_bytes(
        &self,
        conn: &Connection,
        bytes: &[u8],
        created_event_id: i64,
    ) -> AppResult<ObjectHash> {
        let hash = ObjectHash(blake3_hex_prefixed(bytes));
        let path = self.file_path_for_hash(&hash)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_INGEST_READ_FAILED",
                    "object_store",
                    "failed to create object hash prefix directory",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }

        if !path.exists() {
            fs::write(&path, bytes).map_err(|e| {
                AppError::new(
                    "KC_INGEST_READ_FAILED",
                    "object_store",
                    "failed to write object bytes",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
        }

        let relpath = format!("{}/{}", &hash.0[7..9], hash.0);
        conn.execute(
            "INSERT OR IGNORE INTO objects (object_hash, bytes, relpath, created_event_id) VALUES (?1, ?2, ?3, ?4)",
            params![hash.0, bytes.len() as i64, relpath, created_event_id],
        )
        .map_err(|e| {
            AppError::new(
                "KC_DB_INTEGRITY_FAILED",
                "object_store",
                "failed to insert object metadata",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        Ok(hash)
    }

    pub fn get_bytes(&self, object_hash: &ObjectHash) -> AppResult<Vec<u8>> {
        let path = self.file_path_for_hash(object_hash)?;
        fs::read(&path).map_err(|e| {
            AppError::new(
                "KC_INGEST_READ_FAILED",
                "object_store",
                "failed to read object bytes",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })
    }

    pub fn exists(&self, object_hash: &ObjectHash) -> AppResult<bool> {
        Ok(self.file_path_for_hash(object_hash)?.exists())
    }
}
