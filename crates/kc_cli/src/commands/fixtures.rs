use kc_core::app_error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;

fn write_fixture(path: PathBuf, content: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::new(
                "KC_INTERNAL_ERROR",
                "fixtures",
                "failed creating fixture directory",
                false,
                serde_json::json!({ "error": e.to_string(), "path": parent }),
            )
        })?;
    }
    fs::write(&path, content).map_err(|e| {
        AppError::new(
            "KC_INTERNAL_ERROR",
            "fixtures",
            "failed writing fixture file",
            false,
            serde_json::json!({ "error": e.to_string(), "path": path }),
        )
    })
}

pub fn generate_corpus(corpus: &str) -> AppResult<PathBuf> {
    if corpus != "v1" {
        return Err(AppError::new(
            "KC_INTERNAL_ERROR",
            "fixtures",
            "only corpus v1 is currently supported",
            false,
            serde_json::json!({ "corpus": corpus }),
        ));
    }

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let root = workspace_root.join("fixtures").join("golden_corpus").join(corpus);
    write_fixture(
        root.join("md/doc-1.md"),
        br#"# Product Notes

Paragraph A.
## Details
Paragraph B.
"#,
    )?;
    write_fixture(
        root.join("md/doc-2.md"),
        br#"# Meeting

Agenda
## Decisions
Keep deterministic ordering.
"#,
    )?;
    write_fixture(
        root.join("html/page-1.html"),
        br#"<h1>Confluence Root</h1><p>Alpha</p><h2>Section</h2><p>Beta</p>"#,
    )?;
    write_fixture(
        root.join("html/page-2.html"),
        br#"<h1>Another Page</h1><p>Gamma</p><h2>Details</h2><p>Delta</p>"#,
    )?;
    write_fixture(
        root.join("pdf/clean.pdf"),
        b"PDF CLEAN TEXT FIXTURE\nLine one\nLine two\n",
    )?;
    write_fixture(
        root.join("pdf/messy.pdf"),
        b"PDF 123 !!! ??? messy symbols and words",
    )?;
    write_fixture(root.join("pdf/scanned-no-text.pdf"), b"%%%%%")?;

    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::generate_corpus;

    #[test]
    fn fixtures_generate_v1_creates_expected_paths() {
        let root = generate_corpus("v1").expect("generate corpus");
        assert!(root.join("md/doc-1.md").exists());
        assert!(root.join("html/page-1.html").exists());
        assert!(root.join("pdf/clean.pdf").exists());
    }
}
