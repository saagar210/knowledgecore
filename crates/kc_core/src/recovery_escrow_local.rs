use crate::app_error::{AppError, AppResult};
use crate::recovery_escrow::{
    validate_escrow_descriptor, validate_payload_hash, RecoveryEscrowDescriptorV2,
    RecoveryEscrowProvider, RecoveryEscrowProviderStatus, RecoveryEscrowReadRequest,
    RecoveryEscrowWriteRequest,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LocalRecoveryEscrowProvider {
    pub root: PathBuf,
}

impl LocalRecoveryEscrowProvider {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn blob_path(&self, vault_id: &str, payload_hash: &str) -> PathBuf {
        let safe_hash = payload_hash.replace(':', "_");
        self.root.join(vault_id).join(format!("{safe_hash}.enc"))
    }

    fn relative_ref(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

impl RecoveryEscrowProvider for LocalRecoveryEscrowProvider {
    fn provider_id(&self) -> &str {
        "local"
    }

    fn status(&self) -> AppResult<RecoveryEscrowProviderStatus> {
        let configured = !self.root.as_os_str().is_empty();
        let available = configured;
        let details_json = serde_json::to_string(&serde_json::json!({
            "root": self.root,
            "kind": "filesystem",
            "deterministic_path_template": "<vault_id>/<payload_hash>.enc"
        }))
        .map_err(|e| {
            AppError::new(
                "KC_RECOVERY_ESCROW_UNAVAILABLE",
                "recovery",
                "failed serializing local escrow provider status",
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
        if self.root.as_os_str().is_empty() {
            return Err(AppError::new(
                "KC_RECOVERY_ESCROW_UNAVAILABLE",
                "recovery",
                "local recovery escrow root is empty",
                false,
                serde_json::json!({}),
            ));
        }
        let path = self.blob_path(req.vault_id, req.payload_hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_RECOVERY_ESCROW_WRITE_FAILED",
                    "recovery",
                    "failed creating local escrow directory",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }
        fs::write(&path, req.key_blob).map_err(|e| {
            AppError::new(
                "KC_RECOVERY_ESCROW_WRITE_FAILED",
                "recovery",
                "failed writing local escrow payload",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;

        let descriptor = RecoveryEscrowDescriptorV2 {
            provider: self.provider_id().to_string(),
            provider_ref: self.relative_ref(&path),
            key_id: req.payload_hash.to_string(),
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
                "escrow descriptor provider does not match local adapter",
                false,
                serde_json::json!({
                    "expected_provider": self.provider_id(),
                    "actual_provider": req.descriptor.provider
                }),
            ));
        }
        let path = self.root.join(&req.descriptor.provider_ref);
        let bytes = fs::read(&path).map_err(|e| {
            AppError::new(
                "KC_RECOVERY_ESCROW_RESTORE_FAILED",
                "recovery",
                "failed reading local escrow payload",
                false,
                serde_json::json!({ "error": e.to_string(), "path": path }),
            )
        })?;
        validate_payload_hash(&bytes, req.expected_payload_hash)?;
        Ok(bytes)
    }
}
