use crate::embedding::Embedder;
use kc_core::app_error::{AppError, AppResult};
use kc_core::index_traits::{VectorCandidate, VectorIndex};
use kc_core::types::{ChunkId, DocId};

#[derive(Debug, Clone)]
pub struct VectorRow {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub ordinal: i64,
    pub text: String,
    pub vector: Vec<f32>,
}

pub struct InMemoryVectorIndex<E: Embedder> {
    embedder: E,
    rows: Vec<VectorRow>,
}

impl<E: Embedder> InMemoryVectorIndex<E> {
    pub fn new(embedder: E) -> Self {
        Self {
            embedder,
            rows: Vec::new(),
        }
    }

    pub fn upsert_rows(&mut self, rows: Vec<VectorRow>) {
        self.rows = rows;
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

impl<E: Embedder> VectorIndex for InMemoryVectorIndex<E> {
    fn rebuild_for_doc(&self, _doc_id: &DocId) -> AppResult<()> {
        Ok(())
    }

    fn query(&self, query: &str, limit: usize) -> AppResult<Vec<VectorCandidate>> {
        let vectors = self.embedder.embed(&[query.to_string()])?;
        let q = vectors.first().ok_or_else(|| {
            AppError::new(
                "KC_VECTOR_QUERY_FAILED",
                "vector",
                "embedder returned no query vector",
                false,
                serde_json::json!({}),
            )
        })?;

        let mut scored: Vec<(ChunkId, f32)> = self
            .rows
            .iter()
            .map(|row| (row.chunk_id.clone(), cosine_similarity(&row.vector, q)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(idx, (chunk_id, _))| VectorCandidate {
                chunk_id,
                rank: idx as i64 + 1,
            })
            .collect())
    }
}
