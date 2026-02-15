use crate::app_error::{AppError, AppResult};
use crate::recovery_escrow::{
    validate_escrow_descriptor, validate_payload_hash, RecoveryEscrowDescriptorV2,
    RecoveryEscrowProvider, RecoveryEscrowProviderStatus, RecoveryEscrowReadRequest,
    RecoveryEscrowWriteRequest,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GcpRecoveryEscrowConfig {
    pub project_id: String,
    pub location: String,
    pub key_ring: String,
    pub key_name: String,
    pub secret_prefix: String,
}

#[derive(Debug, Clone)]
pub struct GcpRecoveryEscrowProvider {
    pub config: GcpRecoveryEscrowConfig,
}

impl GcpRecoveryEscrowProvider {
    pub fn new(config: GcpRecoveryEscrowConfig) -> Self {
        Self { config }
    }

    fn emulation_root(&self) -> Option<PathBuf> {
        std::env::var("KC_RECOVERY_ESCROW_GCP_EMULATE_DIR")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
    }

    fn emulation_path(&self, root: &Path, vault_id: &str, payload_hash: &str) -> PathBuf {
        let hash = payload_hash.replace(':', "_");
        root.join(vault_id).join(format!("{hash}.enc"))
    }
}

impl RecoveryEscrowProvider for GcpRecoveryEscrowProvider {
    fn provider_id(&self) -> &str {
        "gcp"
    }

    fn status(&self) -> AppResult<RecoveryEscrowProviderStatus> {
        let configured = !self.config.project_id.trim().is_empty()
            && !self.config.location.trim().is_empty()
            && !self.config.key_ring.trim().is_empty()
            && !self.config.key_name.trim().is_empty()
            && !self.config.secret_prefix.trim().is_empty();
        let emulated = self.emulation_root().is_some();
        let available = configured && emulated;

        let details_json = serde_json::to_string(&serde_json::json!({
            "project_id": self.config.project_id,
            "location": self.config.location,
            "key_ring": self.config.key_ring,
            "key_name": self.config.key_name,
            "secret_prefix": self.config.secret_prefix,
            "emulation_enabled": emulated
        }))
        .map_err(|e| {
            AppError::new(
                "KC_RECOVERY_ESCROW_UNAVAILABLE",
                "recovery",
                "failed serializing gcp escrow provider status",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

        Ok(RecoveryEscrowProviderStatus {
            provider: self.provider_id().to_string(),
            configured,
            available,
            details_json,
        })
    }

    fn write(&self, req: RecoveryEscrowWriteRequest<'_>) -> AppResult<RecoveryEscrowDescriptorV2> {
        let Some(root) = self.emulation_root() else {
            return Err(AppError::new(
                "KC_RECOVERY_ESCROW_UNAVAILABLE",
                "recovery",
                "gcp recovery escrow provider is unavailable in this runtime",
                false,
                serde_json::json!({
                    "provider": "gcp",
                    "hint": "configure KC_RECOVERY_ESCROW_GCP_EMULATE_DIR for deterministic local adapter tests"
                }),
            ));
        };
        let path = self.emulation_path(&root, req.vault_id, req.payload_hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_RECOVERY_ESCROW_WRITE_FAILED",
                    "recovery",
                    "failed creating gcp emulation escrow directory",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }
        fs::write(&path, req.key_blob).map_err(|e| {
            AppError::new(
                "KC_RECOVERY_ESCROW_WRITE_FAILED",
                "recovery",
                "failed writing gcp emulation escrow payload",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;

        let provider_ref = path
            .strip_prefix(&root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let descriptor = RecoveryEscrowDescriptorV2 {
            provider: self.provider_id().to_string(),
            provider_ref,
            key_id: format!(
                "gcpkms:{}/{}/{}/{}",
                self.config.project_id, self.config.location, self.config.key_ring, self.config.key_name
            ),
            wrapped_at_ms: req.now_ms,
        };
        validate_escrow_descriptor(&descriptor)?;
        Ok(descriptor)
    }

    fn read(&self, req: RecoveryEscrowReadRequest<'_>) -> AppResult<Vec<u8>> {
        validate_escrow_descriptor(req.descriptor)?;
        if req.descriptor.provider != self.provider_id() {
            return Err(AppError::new(
                "KC_RECOVERY_ESCROW_RESTORE_FAILED",
                "recovery",
                "escrow descriptor provider does not match gcp adapter",
                false,
                serde_json::json!({
                    "expected_provider": self.provider_id(),
                    "actual_provider": req.descriptor.provider
                }),
            ));
        }

        let Some(root) = self.emulation_root() else {
            return Err(AppError::new(
                "KC_RECOVERY_ESCROW_RESTORE_FAILED",
                "recovery",
                "gcp recovery escrow provider is unavailable in this runtime",
                false,
                serde_json::json!({
                    "provider": "gcp",
                    "hint": "configure KC_RECOVERY_ESCROW_GCP_EMULATE_DIR for deterministic local adapter tests"
                }),
            ));
        };

        let path = root.join(&req.descriptor.provider_ref);
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new(
                "KC_RECOVERY_ESCROW_RESTORE_FAILED",
                "recovery",
                "failed reading gcp emulation escrow payload",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;
        validate_payload_hash(&bytes, req.expected_payload_hash)?;
        Ok(bytes)
    }
}
