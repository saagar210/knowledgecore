use crate::embedding::Embedder;
use kc_core::app_error::{AppError, AppResult};
use kc_core::index_traits::{VectorCandidate, VectorIndex};
use kc_core::types::{ChunkId, DocId};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRow {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub ordinal: i64,
    pub text: String,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanceSnapshot {
    schema_version: i64,
    identity: crate::embedding::EmbeddingIdentity,
    rows: Vec<VectorRow>,
}

pub struct LanceDbVectorIndex<E: Embedder> {
    embedder: E,
    db_path: PathBuf,
    rows: Vec<VectorRow>,
    identity: crate::embedding::EmbeddingIdentity,
}

impl<E: Embedder> LanceDbVectorIndex<E> {
    pub fn open(embedder: E, db_path: impl AsRef<Path>) -> AppResult<Self> {
        let identity = embedder.identity();
        let db_path = db_path.as_ref().to_path_buf();
        let mut instance = Self {
            embedder,
            db_path,
            rows: Vec::new(),
            identity,
        };
        instance.load_rows()?;
        Ok(instance)
    }

    pub fn upsert_rows(&mut self, rows: Vec<VectorRow>) -> AppResult<()> {
        self.rows = rows;
        self.persist_rows()
    }

    pub fn embedding_identity(&self) -> &crate::embedding::EmbeddingIdentity {
        &self.identity
    }

    fn load_rows(&mut self) -> AppResult<()> {
        if !self.db_path.exists() {
            return Ok(());
        }
        let bytes = fs::read(&self.db_path).map_err(|e| {
            AppError::new(
                "KC_VECTOR_INDEX_INIT_FAILED",
                "vector",
                "failed reading vector index snapshot",
                false,
                serde_json::json!({ "error": e.to_string(), "path": self.db_path }),
            )
        })?;
        let snapshot: LanceSnapshot = serde_json::from_slice(&bytes).map_err(|e| {
            AppError::new(
                "KC_VECTOR_INDEX_INIT_FAILED",
                "vector",
                "failed parsing vector index snapshot",
                false,
                serde_json::json!({ "error": e.to_string(), "path": self.db_path }),
            )
        })?;
        self.rows = snapshot.rows;
        self.identity = snapshot.identity;
        Ok(())
    }

    fn persist_rows(&self) -> AppResult<()> {
        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::new(
                    "KC_VECTOR_INDEX_INIT_FAILED",
                    "vector",
                    "failed creating vector index directory",
                    false,
                    serde_json::json!({ "error": e.to_string(), "path": parent }),
                )
            })?;
        }

        let snapshot = LanceSnapshot {
            schema_version: 1,
            identity: self.identity.clone(),
            rows: self.rows.clone(),
        };
        let payload = serde_json::to_vec_pretty(&snapshot).map_err(|e| {
            AppError::new(
                "KC_VECTOR_INDEX_INIT_FAILED",
                "vector",
                "failed serializing vector index snapshot",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        fs::write(&self.db_path, payload).map_err(|e| {
            AppError::new(
                "KC_VECTOR_INDEX_INIT_FAILED",
                "vector",
                "failed writing vector index snapshot",
                false,
                serde_json::json!({ "error": e.to_string(), "path": self.db_path }),
            )
        })?;
        Ok(())
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

impl<E: Embedder> VectorIndex for LanceDbVectorIndex<E> {
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

        let mut scored: Vec<(ChunkId, DocId, i64, f32)> = self
            .rows
            .iter()
            .map(|row| {
                (
                    row.chunk_id.clone(),
                    row.doc_id.clone(),
                    row.ordinal,
                    cosine_similarity(&row.vector, q),
                )
            })
            .collect();

        scored.sort_by(|a, b| {
            b.3.partial_cmp(&a.3)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.0.cmp(&b.1.0))
                .then(a.2.cmp(&b.2))
                .then(a.0.0.cmp(&b.0.0))
        });

        Ok(scored
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(idx, (chunk_id, _, _, _))| VectorCandidate {
                chunk_id,
                rank: idx as i64 + 1,
            })
            .collect())
    }
}
