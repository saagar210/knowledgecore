use kc_core::services::{ExtractInput, ExtractService, ToolchainIdentity};
use kc_core::types::DocId;
use kc_extract::DefaultExtractor;

#[test]
fn golden_md_contains_heading_markers() {
    let extractor = DefaultExtractor::new(ToolchainIdentity {
        pdfium_identity: "pdfium:test".to_string(),
        tesseract_identity: "tesseract:test".to_string(),
    });

    let input = b"# Title\n\nhello\n## Child\nworld\n";
    let out = extractor
        .extract_canonical(ExtractInput {
            doc_id: &DocId(
                "blake3:1111111111111111111111111111111111111111111111111111111111111111"
                    .to_string(),
            ),
            bytes: input,
            mime: "text/markdown",
            source_kind: "notes",
        })
        .expect("extract");

    let text = String::from_utf8(out.canonical_bytes).expect("utf8");
    assert!(text.contains("[[H1:Title]]"));
    assert!(text.contains("[[H2:Child]]"));
}

#[test]
fn golden_html_contains_heading_markers() {
    let extractor = DefaultExtractor::new(ToolchainIdentity {
        pdfium_identity: "pdfium:test".to_string(),
        tesseract_identity: "tesseract:test".to_string(),
    });

    let input = b"<h1>Title</h1><p>Hello</p><h2>Child</h2>";
    let out = extractor
        .extract_canonical(ExtractInput {
            doc_id: &DocId(
                "blake3:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_string(),
            ),
            bytes: input,
            mime: "text/html",
            source_kind: "confluence_exports",
        })
        .expect("extract");

    let text = String::from_utf8(out.canonical_bytes).expect("utf8");
    assert!(text.contains("[[H1:Title]]"));
    assert!(text.contains("[[H2:Child]]"));
}
