use crate::verifier::verify_bundle;
use kc_core::app_error::AppResult;
use std::path::Path;

pub fn run_verify(bundle_path: &str) -> AppResult<(i64, crate::verifier::VerifyReportV1)> {
    verify_bundle(Path::new(bundle_path))
}
