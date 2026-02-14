use crate::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultJsonV1 {
    pub schema_version: u32,
    pub vault_id: String,
    pub vault_slug: String,
    pub created_at_ms: i64,
    pub db: VaultDbConfig,
    pub defaults: VaultDefaults,
    pub toolchain: VaultToolchain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDbConfig {
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDefaults {
    pub chunking_config_id: String,
    pub embedding_model_id: String,
    pub recency: VaultRecencyDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRecencyDefaults {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultToolchain {
    pub pdfium: ToolIdentity,
    pub tesseract: ToolIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIdentity {
    pub identity: String,
}

#[derive(Debug, Clone)]
pub struct VaultPaths {
    pub root: PathBuf,
    pub db: PathBuf,
    pub objects_dir: PathBuf,
    pub inbox_dir: PathBuf,
    pub inbox_processed_dir: PathBuf,
    pub vectors_dir: PathBuf,
}

pub fn vault_paths(vault_path: &Path) -> VaultPaths {
    VaultPaths {
        root: vault_path.to_path_buf(),
        db: vault_path.join("db/knowledge.sqlite"),
        objects_dir: vault_path.join("store/objects"),
        inbox_dir: vault_path.join("Inbox"),
        inbox_processed_dir: vault_path.join("Inbox/processed"),
        vectors_dir: vault_path.join("index/vectors"),
    }
}

pub fn vault_init(_vault_path: &Path, _vault_slug: &str, _now_ms: i64) -> AppResult<VaultJsonV1> {
    Err(AppError::internal("vault_init not implemented yet"))
}

pub fn vault_open(_vault_path: &Path) -> AppResult<VaultJsonV1> {
    Err(AppError::internal("vault_open not implemented yet"))
}
