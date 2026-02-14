use kc_core::app_error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingIdentity {
    pub model_id: String,
    pub model_hash: String,
    pub dims: usize,
    pub provider: String,
    pub provider_version: String,
    pub flags_json: String,
}

pub trait Embedder: Send + Sync {
    fn identity(&self) -> EmbeddingIdentity;
    fn embed(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>>;
}
