use crate::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultJsonV2 {
    pub schema_version: u32,
    pub vault_id: String,
    pub vault_slug: String,
    pub created_at_ms: i64,
    pub db: VaultDbConfig,
    pub defaults: VaultDefaults,
    pub toolchain: VaultToolchain,
    #[serde(default)]
    pub encryption: VaultEncryptionConfigV2,
}

// Alias for compatibility with older references in docs/tests.
pub type VaultJsonV1 = VaultJsonV2;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEncryptionConfigV2 {
    pub enabled: bool,
    pub mode: String,
    pub kdf: VaultKdfConfigV2,
    pub key_reference: Option<String>,
}

impl Default for VaultEncryptionConfigV2 {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "object_store_xchacha20poly1305".to_string(),
            kdf: VaultKdfConfigV2::default(),
            key_reference: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultKdfConfigV2 {
    pub algorithm: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub salt_id: String,
}

impl Default for VaultKdfConfigV2 {
    fn default() -> Self {
        Self {
            algorithm: "argon2id".to_string(),
            memory_kib: 65_536,
            iterations: 3,
            parallelism: 1,
            salt_id: "vault-kdf-salt-v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyVaultJsonV1 {
    pub vault_id: String,
    pub vault_slug: String,
    pub created_at_ms: i64,
    pub db: VaultDbConfig,
    pub defaults: VaultDefaults,
    pub toolchain: VaultToolchain,
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

impl VaultJsonV2 {
    pub fn encryption_enabled(&self) -> bool {
        self.encryption.enabled
    }

    pub fn object_store_encryption_context(
        &self,
        passphrase: Option<&str>,
    ) -> AppResult<Option<crate::object_store::ObjectStoreEncryptionContext>> {
        if !self.encryption.enabled {
            return Ok(None);
        }
        if self.encryption.mode != "object_store_xchacha20poly1305" {
            return Err(AppError::new(
                "KC_ENCRYPTION_UNSUPPORTED",
                "encryption",
                "unsupported vault encryption mode",
                false,
                serde_json::json!({
                    "mode": self.encryption.mode,
                    "supported": ["object_store_xchacha20poly1305"]
                }),
            ));
        }
        if self.encryption.kdf.algorithm != "argon2id" {
            return Err(AppError::new(
                "KC_ENCRYPTION_UNSUPPORTED",
                "encryption",
                "unsupported vault kdf algorithm",
                false,
                serde_json::json!({
                    "algorithm": self.encryption.kdf.algorithm,
                    "supported": ["argon2id"]
                }),
            ));
        }

        let passphrase = passphrase.ok_or_else(|| {
            AppError::new(
                "KC_ENCRYPTION_REQUIRED",
                "encryption",
                "vault requires passphrase for encrypted object access",
                false,
                serde_json::json!({}),
            )
        })?;

        let key = crate::object_store::derive_object_store_key(
            passphrase,
            &self.encryption.kdf.salt_id,
            self.encryption.kdf.memory_kib,
            self.encryption.kdf.iterations,
            self.encryption.kdf.parallelism,
        )?;

        Ok(Some(crate::object_store::ObjectStoreEncryptionContext {
            key,
            key_reference: self
                .encryption
                .key_reference
                .clone()
                .unwrap_or_else(|| format!("vault:{}", self.vault_id)),
        }))
    }
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

pub fn vault_init(vault_path: &Path, vault_slug: &str, now_ms: i64) -> AppResult<VaultJsonV2> {
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

    let vault = VaultJsonV2 {
        schema_version: 2,
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
        encryption: VaultEncryptionConfigV2::default(),
    };

    vault_save(vault_path, &vault)?;

    Ok(vault)
}

pub fn vault_save(vault_path: &Path, vault: &VaultJsonV2) -> AppResult<()> {
    let bytes = serde_json::to_vec_pretty(vault).map_err(|e| {
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
    Ok(())
}

pub fn vault_open(vault_path: &Path) -> AppResult<VaultJsonV2> {
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

    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        AppError::new(
            "KC_VAULT_JSON_INVALID",
            "vault",
            "failed to parse vault.json",
            false,
            serde_json::json!({ "error": e.to_string(), "path": path }),
        )
    })?;

    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            AppError::new(
                "KC_VAULT_JSON_INVALID",
                "vault",
                "vault schema_version missing or invalid",
                false,
                serde_json::json!({ "path": path }),
            )
        })? as u32;

    match schema_version {
        1 => {
            let legacy: LegacyVaultJsonV1 = serde_json::from_value(value).map_err(|e| {
                AppError::new(
                    "KC_VAULT_JSON_INVALID",
                    "vault",
                    "failed to parse legacy vault schema v1",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            Ok(VaultJsonV2 {
                schema_version: 2,
                vault_id: legacy.vault_id,
                vault_slug: legacy.vault_slug,
                created_at_ms: legacy.created_at_ms,
                db: legacy.db,
                defaults: legacy.defaults,
                toolchain: legacy.toolchain,
                encryption: VaultEncryptionConfigV2::default(),
            })
        }
        2 => {
            let parsed: VaultJsonV2 = serde_json::from_value(value).map_err(|e| {
                AppError::new(
                    "KC_VAULT_JSON_INVALID",
                    "vault",
                    "failed to parse vault schema v2",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": path }),
                )
            })?;
            if parsed.encryption.enabled && parsed.encryption.mode != "object_store_xchacha20poly1305" {
                return Err(AppError::new(
                    "KC_ENCRYPTION_UNSUPPORTED",
                    "encryption",
                    "vault encryption mode is not supported",
                    false,
                    serde_json::json!({
                        "mode": parsed.encryption.mode,
                        "supported": ["object_store_xchacha20poly1305"]
                    }),
                ));
            }
            Ok(parsed)
        }
        _ => Err(AppError::new(
            "KC_VAULT_JSON_UNSUPPORTED_VERSION",
            "vault",
            "unsupported vault schema_version",
            false,
            serde_json::json!({ "expected": [1, 2], "actual": schema_version }),
        )),
    }
}
