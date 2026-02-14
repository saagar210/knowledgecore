use kc_core::app_error::AppResult;
use kc_core::deferred;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct PreviewStatusPayload {
    schema_version: i64,
    status: String,
    capabilities: Vec<deferred::DraftCapabilityStatusV1>,
}

pub fn run_status() -> AppResult<()> {
    let payload = PreviewStatusPayload {
        schema_version: 1,
        status: "draft".to_string(),
        capabilities: deferred::preview_capability_statuses(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

pub fn run_capability(name: &str) -> AppResult<()> {
    Err(deferred::scaffold_error_for_capability(name))
}
