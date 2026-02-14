use crate::app_error::AppResult;
use crate::types::{ChunkId, DocId};

#[derive(Debug, Clone)]
pub struct LexicalCandidate {
    pub chunk_id: ChunkId,
    pub rank: i64,
}

#[derive(Debug, Clone)]
pub struct VectorCandidate {
    pub chunk_id: ChunkId,
    pub rank: i64,
}

pub trait LexicalIndex: Send + Sync {
    fn rebuild_for_doc(&self, doc_id: &DocId) -> AppResult<()>;
    fn query(&self, query: &str, limit: usize) -> AppResult<Vec<LexicalCandidate>>;
}

pub trait VectorIndex: Send + Sync {
    fn rebuild_for_doc(&self, doc_id: &DocId) -> AppResult<()>;
    fn query(&self, query: &str, limit: usize) -> AppResult<Vec<VectorCandidate>>;
}
