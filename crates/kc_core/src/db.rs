use crate::app_error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;

pub fn open_db(_db_path: &Path) -> AppResult<Connection> {
    Err(AppError::internal("open_db not implemented yet"))
}

pub fn apply_migrations(_conn: &Connection) -> AppResult<()> {
    Err(AppError::internal("apply_migrations not implemented yet"))
}

pub fn schema_version(_conn: &Connection) -> AppResult<i64> {
    Err(AppError::internal("schema_version not implemented yet"))
}
