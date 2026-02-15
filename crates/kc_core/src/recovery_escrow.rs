use crate::app_error::{AppError, AppResult};
use crate::canon_json::to_canonical_bytes;
use crate::hashing::blake3_hex_prefixed;
use serde::{Deserialize, Serialize};

pub const ESCROW_PROVIDER_PRIORITY: [&str; 4] = ["aws", "gcp", "azure", "local"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryEscrowDescriptorV2 {
    pub provider: String,
    pub provider_ref: String,
    pub key_id: String,
    pub wrapped_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryEscrowProviderConfigV3 {
    pub provider_id: String,
    pub config_ref: String,
    pub enabled: bool,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryEscrowProviderStatus {
    pub provider: String,
    pub configured: bool,
    pub available: bool,
    pub details_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEscrowWriteRequest<'a> {
    pub vault_id: &'a str,
    pub payload_hash: &'a str,
    pub key_blob: &'a [u8],
    pub now_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEscrowReadRequest<'a> {
    pub descriptor: &'a RecoveryEscrowDescriptorV2,
    pub expected_payload_hash: &'a str,
}

pub trait RecoveryEscrowProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn status(&self) -> AppResult<RecoveryEscrowProviderStatus>;
    fn write(&self, req: RecoveryEscrowWriteRequest<'_>) -> AppResult<RecoveryEscrowDescriptorV2>;
    fn read(&self, req: RecoveryEscrowReadRequest<'_>) -> AppResult<Vec<u8>>;
}

pub fn provider_priority(provider_id: &str) -> i64 {
    ESCROW_PROVIDER_PRIORITY
        .iter()
        .position(|candidate| *candidate == provider_id)
        .map(|idx| idx as i64)
        .unwrap_or(9)
}

pub fn supported_provider_ids() -> &'static [&'static str] {
    &ESCROW_PROVIDER_PRIORITY
}

pub fn normalize_provider_configs(configs: &mut [RecoveryEscrowProviderConfigV3]) {
    configs.sort_by(|a, b| {
        provider_priority(&a.provider_id)
            .cmp(&provider_priority(&b.provider_id))
            .then_with(|| a.provider_id.cmp(&b.provider_id))
            .then_with(|| a.config_ref.cmp(&b.config_ref))
    });
}

pub fn normalize_escrow_descriptors(descs: &mut [RecoveryEscrowDescriptorV2]) {
    descs.sort_by(|a, b| {
        provider_priority(&a.provider)
            .cmp(&provider_priority(&b.provider))
            .then_with(|| a.provider.cmp(&b.provider))
            .then_with(|| a.provider_ref.cmp(&b.provider_ref))
            .then_with(|| a.key_id.cmp(&b.key_id))
            .then_with(|| a.wrapped_at_ms.cmp(&b.wrapped_at_ms))
    });
}

pub fn validate_escrow_descriptor(desc: &RecoveryEscrowDescriptorV2) -> AppResult<()> {
    if desc.provider.trim().is_empty()
        || desc.provider_ref.trim().is_empty()
        || desc.key_id.trim().is_empty()
    {
        return Err(AppError::new(
            "KC_RECOVERY_ESCROW_WRITE_FAILED",
            "recovery",
            "escrow descriptor contains empty required fields",
            false,
            serde_json::json!({
                "provider": desc.provider,
                "provider_ref": desc.provider_ref,
                "key_id": desc.key_id
            }),
        ));
    }
    Ok(())
}

pub fn canonical_descriptor_hash(desc: &RecoveryEscrowDescriptorV2) -> AppResult<String> {
    validate_escrow_descriptor(desc)?;
    let value = serde_json::to_value(desc).map_err(|e| {
        AppError::new(
            "KC_RECOVERY_ESCROW_WRITE_FAILED",
            "recovery",
            "failed serializing escrow descriptor",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let canonical = to_canonical_bytes(&value)?;
    Ok(blake3_hex_prefixed(&canonical))
}

pub fn validate_payload_hash(bytes: &[u8], expected_payload_hash: &str) -> AppResult<()> {
    let actual = blake3_hex_prefixed(bytes);
    if actual != expected_payload_hash {
        return Err(AppError::new(
            "KC_RECOVERY_ESCROW_RESTORE_FAILED",
            "recovery",
            "escrow payload hash mismatch",
            false,
            serde_json::json!({
                "expected": expected_payload_hash,
                "actual": actual
            }),
        ));
    }
    Ok(())
}
