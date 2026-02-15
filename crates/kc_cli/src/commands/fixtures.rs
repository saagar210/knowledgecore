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

fn escape_pdf_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn build_single_page_pdf(lines: &[&str]) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    let mut offsets = [0usize; 6];

    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let content_stream = if lines.is_empty() {
        "BT\n/F1 12 Tf\n72 720 Td\nET\n".to_string()
    } else {
        let mut stream = String::from("BT\n/F1 12 Tf\n72 720 Td\n");
        for (idx, line) in lines.iter().enumerate() {
            if idx > 0 {
                stream.push_str("0 -16 Td\n");
            }
            stream.push_str(&format!("({}) Tj\n", escape_pdf_text(line)));
        }
        stream.push_str("ET\n");
        stream
    };

    let objects = vec![
        (1, "<< /Type /Catalog /Pages 2 0 R >>".to_string()),
        (2, "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string()),
        (
            3,
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        ),
        (4, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string()),
        (
            5,
            format!(
                "<< /Length {} >>\nstream\n{}endstream",
                content_stream.len(),
                content_stream
            ),
        ),
    ];

    for (id, body) in objects {
        offsets[id] = out.len();
        out.extend_from_slice(format!("{id} 0 obj\n{body}\nendobj\n").as_bytes());
    }

    let xref_start = out.len();
    out.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            xref_start
        )
        .as_bytes(),
    );

    out
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
    let root = workspace_root
        .join("fixtures")
        .join("golden_corpus")
        .join(corpus);
    write_fixture(
        root.join("md/doc-1.md"),
        br#"# Product Notes

This corpus validates deterministic extraction.

## Details

Paragraph B with punctuation, numbers (123), and symbols.

### Subsection

Nested heading content to verify marker depth handling.
"#,
    )?;
    write_fixture(
        root.join("md/doc-2.md"),
        br#"# Meeting Notes

Agenda item one.

## Decisions

Keep deterministic ordering.
Record explicit assumptions.

## Follow Ups

Test OCR + PDF extraction boundaries.
"#,
    )?;
    write_fixture(
        root.join("html/page-1.html"),
        br#"<h1>Confluence Root</h1><p>Alpha paragraph for extraction baseline.</p><h2>Section</h2><p>Beta details.</p><h3>Nested</h3><p>Gamma nested text.</p>"#,
    )?;
    write_fixture(
        root.join("html/page-2.html"),
        br#"<h1>Another Page</h1><p>Gamma content.</p><h2>Details</h2><p>Delta details plus 456.</p>"#,
    )?;
    write_fixture(
        root.join("pdf/clean.pdf"),
        &build_single_page_pdf(&[
            "Clean PDF content line one.",
            "Line two keeps deterministic output.",
            "Line three includes 2026 references.",
        ]),
    )?;
    write_fixture(
        root.join("pdf/messy.pdf"),
        &build_single_page_pdf(&[
            "Messy !!! ??? symbols ###",
            "Mixed123Text and punctuation",
            "Spacing   variations",
        ]),
    )?;
    write_fixture(
        root.join("pdf/scanned-no-text.pdf"),
        &build_single_page_pdf(&[]),
    )?;

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
        let pdf_bytes = std::fs::read(root.join("pdf/clean.pdf")).expect("read pdf");
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
    }
}
