use crate::app_error::{AppError, AppResult};
use crate::canon_json::hash_canonical;
use crate::hashing::blake3_hex_prefixed;
use crate::types::{ChunkId, ConfigHash, DocId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfigV1 {
    pub v: i64,
    pub md_html: MdHtmlChunkCfg,
    pub pdf: PdfChunkCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdHtmlChunkCfg {
    pub max_chars: usize,
    pub min_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfChunkCfg {
    pub window_chars: usize,
    pub overlap_chars: usize,
    pub respect_markers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub ordinal: i64,
    pub start_char: i64,
    pub end_char: i64,
    pub chunking_config_hash: ConfigHash,
}

pub fn hash_chunking_config(cfg: &ChunkingConfigV1) -> AppResult<ConfigHash> {
    let value = serde_json::to_value(cfg).map_err(|e| {
        AppError::new(
            "KC_CHUNK_CONFIG_INVALID",
            "chunking",
            "chunk config must be serializable",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    Ok(ConfigHash(hash_canonical(&value)?))
}

fn build_chunk_id(doc_id: &DocId, cfg_hash: &ConfigHash, ordinal: i64, start: i64, end: i64) -> ChunkId {
    let raw = format!(
        "kc.chunk.v1\n{}\n{}\n{}\n{}:{}",
        doc_id.0, cfg_hash.0, ordinal, start, end
    );
    ChunkId(blake3_hex_prefixed(raw.as_bytes()))
}

pub fn chunk_document(doc_id: &DocId, canonical_text: &str, mime: &str, cfg: &ChunkingConfigV1) -> AppResult<Vec<ChunkRecord>> {
    if cfg.v != 1 {
        return Err(AppError::new(
            "KC_CHUNK_CONFIG_INVALID",
            "chunking",
            "unsupported chunking config version",
            false,
            serde_json::json!({ "expected": 1, "actual": cfg.v }),
        ));
    }

    let cfg_hash = hash_chunking_config(cfg)?;
    let total = canonical_text.chars().count() as i64;
    if total == 0 {
        return Ok(Vec::new());
    }

    let mut chunks = Vec::new();

    if mime == "application/pdf" {
        let window = cfg.pdf.window_chars.max(1) as i64;
        let overlap = cfg.pdf.overlap_chars.min(cfg.pdf.window_chars.saturating_sub(1)) as i64;
        let step = (window - overlap).max(1);
        let mut start = 0i64;
        let mut ordinal = 0i64;

        while start < total {
            let end = (start + window).min(total);
            chunks.push(ChunkRecord {
                chunk_id: build_chunk_id(doc_id, &cfg_hash, ordinal, start, end),
                doc_id: doc_id.clone(),
                ordinal,
                start_char: start,
                end_char: end,
                chunking_config_hash: cfg_hash.clone(),
            });
            if end == total {
                break;
            }
            start += step;
            ordinal += 1;
        }
    } else {
        let max = cfg.md_html.max_chars.max(1) as i64;
        let min = cfg.md_html.min_chars.min(cfg.md_html.max_chars).max(1) as i64;
        let mut start = 0i64;
        let mut ordinal = 0i64;

        while start < total {
            let mut end = (start + max).min(total);
            if total - end < min && end != total {
                end = total;
            }
            chunks.push(ChunkRecord {
                chunk_id: build_chunk_id(doc_id, &cfg_hash, ordinal, start, end),
                doc_id: doc_id.clone(),
                ordinal,
                start_char: start,
                end_char: end,
                chunking_config_hash: cfg_hash.clone(),
            });

            if end == total {
                break;
            }
            start = end;
            ordinal += 1;
        }
    }

    Ok(chunks)
}
