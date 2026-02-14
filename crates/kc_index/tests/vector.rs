use kc_core::app_error::AppResult;
use kc_core::index_traits::VectorIndex;
use kc_core::types::{ChunkId, DocId};
use kc_index::embedding::{Embedder, EmbeddingIdentity};
use kc_index::vector::{LanceDbVectorIndex, VectorRow};

struct DummyEmbedder;

impl Embedder for DummyEmbedder {
    fn identity(&self) -> EmbeddingIdentity {
        EmbeddingIdentity {
            model_id: "dummy".to_string(),
            model_hash: "blake3:dummy".to_string(),
            dims: 2,
            provider: "test".to_string(),
            provider_version: "1".to_string(),
            flags_json: "{}".to_string(),
        }
    }

    fn embed(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| if t.contains("alpha") { vec![1.0, 0.0] } else { vec![0.0, 1.0] })
            .collect())
    }
}

#[test]
fn vector_query_returns_ranked_hits() {
    let db_path = tempfile::tempdir().expect("tempdir").keep().join("vectors/lancedb.json");
    let mut index = LanceDbVectorIndex::open(DummyEmbedder, &db_path).expect("open index");
    index
        .upsert_rows(vec![
        VectorRow {
            chunk_id: ChunkId("c1".to_string()),
            doc_id: DocId("d1".to_string()),
            ordinal: 0,
            text: "alpha text".to_string(),
            vector: vec![1.0, 0.0],
        },
        VectorRow {
            chunk_id: ChunkId("c2".to_string()),
            doc_id: DocId("d2".to_string()),
            ordinal: 0,
            text: "beta text".to_string(),
            vector: vec![0.0, 1.0],
        },
        ])
        .expect("upsert rows");

    let hits = index.query("alpha", 10).expect("query");
    assert_eq!(hits[0].chunk_id.0, "c1");
    assert_eq!(index.embedding_identity().model_id, "dummy");

    let reloaded = LanceDbVectorIndex::open(DummyEmbedder, &db_path).expect("reload index");
    let reloaded_hits = reloaded.query("alpha", 10).expect("query from reloaded index");
    assert_eq!(reloaded_hits[0].chunk_id.0, "c1");
}
