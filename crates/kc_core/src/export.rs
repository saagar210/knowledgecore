use crate::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_vectors: bool,
}

pub fn export_bundle(_vault_path: &Path, _export_dir: &Path, _opts: &ExportOptions, _now_ms: i64) -> AppResult<PathBuf> {
    Err(AppError::internal("export_bundle not implemented yet"))
}
