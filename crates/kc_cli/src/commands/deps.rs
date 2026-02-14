use kc_core::app_error::{AppError, AppResult};

fn check_cmd(cmd: &str, code: &str) -> AppResult<()> {
    let ok = std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(AppError::new(
            code,
            "deps",
            "required dependency is unavailable",
            true,
            serde_json::json!({ "command": cmd }),
        ))
    }
}

pub fn run_check() -> AppResult<()> {
    check_cmd("pdfium", "KC_PDFIUM_UNAVAILABLE")?;
    check_cmd("tesseract", "KC_TESSERACT_UNAVAILABLE")?;
    println!("dependency check ok");
    Ok(())
}
