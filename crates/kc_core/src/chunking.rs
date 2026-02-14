use crate::app_error::{AppError, AppResult};
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

pub fn hash_chunking_config(_cfg: &ChunkingConfigV1) -> AppResult<ConfigHash> {
    Err(AppError::internal("hash_chunking_config not implemented yet"))
}

pub fn chunk_document(_doc_id: &DocId, _canonical_text: &str, _mime: &str, _cfg: &ChunkingConfigV1) -> AppResult<Vec<ChunkRecord>> {
    Err(AppError::internal("chunk_document not implemented yet"))
}
