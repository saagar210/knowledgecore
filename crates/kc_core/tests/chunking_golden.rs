use kc_core::chunking::{
    chunk_document, hash_chunking_config, ChunkingConfigV1, MdHtmlChunkCfg, PdfChunkCfg,
};
use kc_core::types::DocId;

fn cfg() -> ChunkingConfigV1 {
    ChunkingConfigV1 {
        v: 1,
        md_html: MdHtmlChunkCfg {
            max_chars: 10,
            min_chars: 4,
        },
        pdf: PdfChunkCfg {
            window_chars: 8,
            overlap_chars: 2,
            respect_markers: true,
        },
    }
}

#[test]
fn chunking_golden_md_html_deterministic() {
    let text = "abcdefghijklmnopqrstuvwxyz";
    let chunks = chunk_document(
        &DocId(
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ),
        text,
        "text/markdown",
        &cfg(),
    )
    .expect("chunk");

    let ranges: Vec<(i64, i64)> = chunks.iter().map(|c| (c.start_char, c.end_char)).collect();
    assert_eq!(ranges, vec![(0, 10), (10, 20), (20, 26)]);
}

#[test]
fn chunking_golden_pdf_window_overlap() {
    let text = "abcdefghijklmnopqrstuvwxyz";
    let chunks = chunk_document(
        &DocId(
            "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        ),
        text,
        "application/pdf",
        &cfg(),
    )
    .expect("chunk");

    let ranges: Vec<(i64, i64)> = chunks.iter().map(|c| (c.start_char, c.end_char)).collect();
    assert_eq!(ranges, vec![(0, 8), (6, 14), (12, 20), (18, 26)]);
}

#[test]
fn chunking_config_hash_stable() {
    let h1 = hash_chunking_config(&cfg()).expect("hash");
    let h2 = hash_chunking_config(&cfg()).expect("hash");
    assert_eq!(h1.0, h2.0);
}
