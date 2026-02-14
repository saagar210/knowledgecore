use kc_core::app_error::{AppError, AppResult};
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
struct ToolStatus {
    command: String,
    available: bool,
    version: Option<String>,
}

fn probe(cmd: &str, args: &[&str]) -> ToolStatus {
    match Command::new(cmd).args(args).output() {
        Ok(out) => {
            let output = if out.stdout.is_empty() {
                String::from_utf8_lossy(&out.stderr).to_string()
            } else {
                String::from_utf8_lossy(&out.stdout).to_string()
            };
            ToolStatus {
                command: cmd.to_string(),
                available: out.status.success(),
                version: Some(output.lines().next().unwrap_or_default().to_string()),
            }
        }
        Err(_) => ToolStatus {
            command: cmd.to_string(),
            available: false,
            version: None,
        },
    }
}

pub fn run_check() -> AppResult<()> {
    let pdf_text = probe("pdftotext", &["-v"]);
    let pdf_render = probe("pdftoppm", &["-v"]);
    let tesseract = probe("tesseract", &["--version"]);

    if !pdf_text.available || !pdf_render.available {
        return Err(AppError::new(
            "KC_PDFIUM_UNAVAILABLE",
            "deps",
            "pdf backend tools are unavailable",
            true,
            serde_json::json!({
                "required": ["pdftotext", "pdftoppm"],
                "status": [pdf_text, pdf_render]
            }),
        ));
    }

    if !tesseract.available {
        return Err(AppError::new(
            "KC_TESSERACT_UNAVAILABLE",
            "deps",
            "tesseract is unavailable",
            true,
            serde_json::json!({
                "required": ["tesseract"],
                "status": [tesseract]
            }),
        ));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "tools": [pdf_text, pdf_render, tesseract]
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::probe;

    #[test]
    fn probe_reports_unavailable_for_unknown_command() {
        let status = probe("kc_command_that_should_not_exist", &["--version"]);
        assert!(!status.available);
        assert!(status.version.is_none());
    }
}
