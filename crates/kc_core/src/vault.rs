use crate::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

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

pub fn vault_init(vault_path: &Path, vault_slug: &str, now_ms: i64) -> AppResult<VaultJsonV1> {
    let paths = vault_paths(vault_path);
    fs::create_dir_all(paths.db.parent().ok_or_else(|| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "unable to resolve db parent directory",
            false,
            serde_json::json!({ "vault_path": vault_path }),
        )
    })?)
    .map_err(|e| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "failed to create db directory",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    fs::create_dir_all(&paths.objects_dir).map_err(|e| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "failed to create objects directory",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    fs::create_dir_all(&paths.inbox_processed_dir).map_err(|e| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "failed to create inbox processed directory",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    fs::create_dir_all(&paths.vectors_dir).map_err(|e| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "failed to create vectors directory",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let vault = VaultJsonV1 {
        schema_version: 1,
        vault_id: Uuid::new_v4().to_string(),
        vault_slug: vault_slug.to_string(),
        created_at_ms: now_ms,
        db: VaultDbConfig {
            relative_path: "db/knowledge.sqlite".to_string(),
        },
        defaults: VaultDefaults {
            chunking_config_id: "chunking/default-v1".to_string(),
            embedding_model_id: "embedding/default-v1".to_string(),
            recency: VaultRecencyDefaults { enabled: false },
        },
        toolchain: VaultToolchain {
            pdfium: ToolIdentity {
                identity: "pdfium:unconfigured".to_string(),
            },
            tesseract: ToolIdentity {
                identity: "tesseract:unconfigured".to_string(),
            },
        },
    };

    let bytes = serde_json::to_vec_pretty(&vault).map_err(|e| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "failed to serialize vault.json",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    fs::write(vault_path.join("vault.json"), bytes).map_err(|e| {
        AppError::new(
            "KC_VAULT_INIT_FAILED",
            "vault",
            "failed to write vault.json",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    Ok(vault)
}

pub fn vault_open(vault_path: &Path) -> AppResult<VaultJsonV1> {
    let path = vault_path.join("vault.json");
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::new(
                "KC_VAULT_JSON_MISSING",
                "vault",
                "vault.json is missing",
                false,
                serde_json::json!({ "path": path }),
            )
        } else {
            AppError::new(
                "KC_VAULT_JSON_INVALID",
                "vault",
                "failed to read vault.json",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        }
    })?;

    let parsed: VaultJsonV1 = serde_json::from_slice(&bytes).map_err(|e| {
        AppError::new(
            "KC_VAULT_JSON_INVALID",
            "vault",
            "failed to parse vault.json",
            false,
            serde_json::json!({ "error": e.to_string(), "path": path }),
        )
    })?;

    if parsed.schema_version != 1 {
        return Err(AppError::new(
            "KC_VAULT_JSON_UNSUPPORTED_VERSION",
            "vault",
            "unsupported vault schema_version",
            false,
            serde_json::json!({ "expected": 1, "actual": parsed.schema_version }),
        ));
    }

    Ok(parsed)
}
