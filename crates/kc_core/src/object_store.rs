use crate::app_error::{AppError, AppResult};
use crate::types::ObjectHash;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct ObjectStore {
    pub objects_dir: PathBuf,
}

impl ObjectStore {
    pub fn new(objects_dir: PathBuf) -> Self {
        Self { objects_dir }
    }

    pub fn put_bytes(&self, _conn: &Connection, _bytes: &[u8], _created_event_id: i64) -> AppResult<ObjectHash> {
        Err(AppError::internal("put_bytes not implemented yet"))
    }

    pub fn get_bytes(&self, _object_hash: &ObjectHash) -> AppResult<Vec<u8>> {
        Err(AppError::internal("get_bytes not implemented yet"))
    }

    pub fn exists(&self, _object_hash: &ObjectHash) -> AppResult<bool> {
        Err(AppError::internal("exists not implemented yet"))
    }
}
