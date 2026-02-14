use kc_core::app_error::AppError;
use kc_core::vault::vault_init;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcResponse<T> {
    Ok { ok: bool, data: T },
    Err { ok: bool, error: AppError },
}

impl<T> RpcResponse<T> {
    pub fn ok(data: T) -> Self {
        Self::Ok { ok: true, data }
    }

    pub fn err(error: AppError) -> Self {
        Self::Err { ok: false, error }
    }
}

#[derive(Debug, Deserialize)]
pub struct VaultInitReq {
    pub vault_path: String,
    pub vault_slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInitRes {
    pub vault_id: String,
}

pub fn vault_init_rpc(req: VaultInitReq, now_ms: i64) -> RpcResponse<VaultInitRes> {
    match vault_init(std::path::Path::new(&req.vault_path), &req.vault_slug, now_ms) {
        Ok(vault) => RpcResponse::ok(VaultInitRes {
            vault_id: vault.vault_id,
        }),
        Err(error) => RpcResponse::err(error),
    }
}
