use kc_core::app_error::AppResult;
use kc_core::export::{export_bundle, ExportOptions};
use std::path::Path;

pub fn run_export(vault_path: &str, export_dir: &str, now_ms: i64) -> AppResult<std::path::PathBuf> {
    export_bundle(
        Path::new(vault_path),
        Path::new(export_dir),
        &ExportOptions {
            include_vectors: false,
        },
        now_ms,
    )
}
