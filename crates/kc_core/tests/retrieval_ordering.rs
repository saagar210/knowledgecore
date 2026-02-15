use kc_core::index_traits::{LexicalCandidate, VectorCandidate};
use kc_core::retrieval::{merge_candidates, RecencyConfigV1, RetrievalConfigV1};
use kc_core::types::{ChunkId, DocId};

#[test]
fn retrieval_ordering_is_deterministic_with_tiebreaks() {
    let lexical = vec![
        LexicalCandidate {
            chunk_id: ChunkId("c1".to_string()),
            rank: 1,
        },
        LexicalCandidate {
            chunk_id: ChunkId("c2".to_string()),
            rank: 2,
        },
    ];

    let vector = vec![
        VectorCandidate {
            chunk_id: ChunkId("c2".to_string()),
            rank: 1,
        },
        VectorCandidate {
            chunk_id: ChunkId("c1".to_string()),
            rank: 2,
        },
    ];

    let cfg = RetrievalConfigV1 {
        rrf_k: 60,
        w_lex: 1.0,
        w_vec: 1.0,
        recency: RecencyConfigV1 {
            enabled: false,
            window_days: 30,
            max_boost: 0.03,
        },
    };

    let hits = merge_candidates(
        &lexical,
        &vector,
        |chunk_id| {
            let (doc, ord) = match chunk_id.0.as_str() {
                "c1" => ("d1", 0),
                "c2" => ("d2", 0),
                _ => ("d3", 0),
            };
            Ok((DocId(doc.to_string()), ord, "other".to_string(), 0))
        },
        &cfg,
        0,
    )
    .expect("merge");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].chunk_id.0, "c1");
    assert_eq!(hits[1].chunk_id.0, "c2");
}

#[test]
fn retrieval_recency_changes_order_when_enabled() {
    let lexical = vec![LexicalCandidate {
        chunk_id: ChunkId("c_old".to_string()),
        rank: 1,
    }];

    let vector = vec![VectorCandidate {
        chunk_id: ChunkId("c_new".to_string()),
        rank: 1,
    }];

    let cfg = RetrievalConfigV1 {
        rrf_k: 60,
        w_lex: 1.0,
        w_vec: 1.0,
        recency: RecencyConfigV1 {
            enabled: true,
            window_days: 30,
            max_boost: 0.03,
        },
    };

    let now_ms = 1_700_000_000_000i64;
    let hits = merge_candidates(
        &lexical,
        &vector,
        |chunk_id| {
            if chunk_id.0 == "c_old" {
                Ok((
                    DocId("d1".to_string()),
                    0,
                    "other".to_string(),
                    now_ms - 40 * 24 * 60 * 60 * 1000,
                ))
            } else {
                Ok((
                    DocId("d2".to_string()),
                    0,
                    "other".to_string(),
                    now_ms - 2 * 24 * 60 * 60 * 1000,
                ))
            }
        },
        &cfg,
        now_ms,
    )
    .expect("merge");

    assert_eq!(hits[0].chunk_id.0, "c_new");
}
