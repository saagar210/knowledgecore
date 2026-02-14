use kc_core::app_error::AppResult;
use kc_core::rpc_service::{
    trust_device_enroll_service, trust_device_list_service, trust_device_verify_chain_service,
    trust_identity_complete_service, trust_identity_start_service,
};
use std::path::Path;

fn now_ms() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch");
    now.as_millis() as i64
}

pub fn run_identity_start(vault_path: &str, provider: &str, now_override: Option<i64>) -> AppResult<()> {
    let out = trust_identity_start_service(
        Path::new(vault_path),
        provider,
        now_override.unwrap_or_else(now_ms),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "identity": out
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_identity_complete(
    vault_path: &str,
    provider: &str,
    code: &str,
    now_override: Option<i64>,
) -> AppResult<()> {
    let out = trust_identity_complete_service(
        Path::new(vault_path),
        provider,
        code,
        now_override.unwrap_or_else(now_ms),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "session": out
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_device_enroll(
    vault_path: &str,
    device_label: &str,
    now_override: Option<i64>,
) -> AppResult<()> {
    let out = trust_device_enroll_service(
        Path::new(vault_path),
        device_label,
        now_override.unwrap_or_else(now_ms),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "device": out.device,
            "certificate": out.certificate
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_device_verify_chain(
    vault_path: &str,
    device_id: &str,
    now_override: Option<i64>,
) -> AppResult<()> {
    let out = trust_device_verify_chain_service(
        Path::new(vault_path),
        device_id,
        now_override.unwrap_or_else(now_ms),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "certificate": out
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_device_list(vault_path: &str) -> AppResult<()> {
    let out = trust_device_list_service(Path::new(vault_path))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "devices": out
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}
